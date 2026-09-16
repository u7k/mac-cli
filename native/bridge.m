#import <Foundation/Foundation.h>
#import <AppKit/AppKit.h>
#import <CoreAudio/CoreAudio.h>
#import <CoreWLAN/CoreWLAN.h>
#import <CoreLocation/CoreLocation.h>
#import <CoreGraphics/CoreGraphics.h>
#import <EventKit/EventKit.h>
#import <IOKit/ps/IOPowerSources.h>
#import <IOKit/ps/IOPSKeys.h>
#import <IOKit/IOKitLib.h>
#import <IOBluetooth/IOBluetooth.h>
#import <Carbon/Carbon.h>
#import <Vision/Vision.h>
#include <dlfcn.h>
#include <math.h>
#include <ifaddrs.h>
#include <netdb.h>

static NSNumber *boolean(BOOL value) { return value ? @YES : @NO; }
static id nilValue(id v) { return v ?: [NSNull null]; }
static NSDictionary *good(id data) { return @{@"data":nilValue(data)}; }
static NSDictionary *bad(NSString *code, NSString *message) { return @{@"error":@{@"code":code,@"message":message}}; }
static NSDictionary *unsupported(NSString *feature) { return bad(@"unsupported", [feature stringByAppendingString:@" is unavailable on this device or macOS release."]); }
static NSDictionary *osError(NSString *action, NSInteger code) {
    return bad(@"system_error", [NSString stringWithFormat:@"%@ failed (macOS error %ld).",action,(long)code]);
}
static BOOL pumpUntil(BOOL (^done)(void), NSTimeInterval timeout) {
    NSDate *limit = [NSDate dateWithTimeIntervalSinceNow:timeout];
    while (!done() && limit.timeIntervalSinceNow > 0) {
        [[NSRunLoop currentRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.02]];
    }
    return done();
}
static void settle(void) { [[NSRunLoop currentRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.15]]; }
static double adjusted(NSDictionary *a, double old) {
    NSString *action = a[@"action"] ?: @"get";
    if ([action isEqual:@"set"]) return [a[@"value"] doubleValue];
    if ([action isEqual:@"up"]) return fmin(100, old + [a[@"value"] doubleValue]);
    if ([action isEqual:@"down"]) return fmax(0, old - [a[@"value"] doubleValue]);
    return old;
}

static NSDictionary *battery(void) {
    CFTypeRef info = IOPSCopyPowerSourcesInfo();
    if (!info) return unsupported(@"Battery information");
    NSArray *sources = CFBridgingRelease(IOPSCopyPowerSourcesList(info));
    NSDictionary *battery = nil;
    for (id source in sources) {
        NSDictionary *item = (__bridge NSDictionary *)IOPSGetPowerSourceDescription(info, (__bridge CFTypeRef)source);
        if ([item[@kIOPSTypeKey] isEqual:@kIOPSInternalBatteryType]) { battery = [item copy]; break; }
    }
    CFRelease(info);
    if (!battery) return unsupported(@"An internal battery");
    NSNumber *current = battery[@kIOPSCurrentCapacityKey], *maximum = battery[@kIOPSMaxCapacityKey];
    id percent = maximum.doubleValue > 0 ? @(100.0 * current.doubleValue / maximum.doubleValue) : [NSNull null];
    BOOL charging = [battery[@kIOPSIsChargingKey] boolValue];
    NSString *state = charging ? @"Charging" : ([battery[@kIOPSPowerSourceStateKey] isEqual:@kIOPSACPowerValue] ? @"External power" : @"Discharging");
    NSMutableDictionary *data = [@{@"title":@"Battery", @"percent":percent, @"state":state, @"power":nilValue(battery[@kIOPSPowerSourceStateKey])} mutableCopy];
    NSNumber *minutes = battery[charging ? @kIOPSTimeToFullChargeKey : @kIOPSTimeToEmptyKey];
    data[@"remaining_minutes"] = minutes.intValue > 0 ? minutes : [NSNull null];
    io_service_t service = IOServiceGetMatchingService(kIOMainPortDefault,IOServiceMatching("AppleSmartBattery"));
    if (service) {
        CFMutableDictionaryRef props = NULL;
        if (IORegistryEntryCreateCFProperties(service,&props,kCFAllocatorDefault,0) == KERN_SUCCESS) {
            NSDictionary *p = CFBridgingRelease(props);
            data[@"cycle_count"] = nilValue(p[@"CycleCount"]);
            data[@"design_capacity_mah"] = nilValue(p[@"DesignCapacity"]);
            data[@"maximum_capacity_mah"] = nilValue(p[@"AppleRawMaxCapacity"]);
            data[@"health"] = nilValue(battery[@"BatteryHealth"]);
        }
        IOObjectRelease(service);
    }
    return good(data);
}

typedef int (*BrightnessGet)(CGDirectDisplayID, float *);
typedef int (*BrightnessSet)(CGDirectDisplayID, float);
static void *displayLibrary(void) {
    static void *handle;
    if (!handle) handle = dlopen("/System/Library/PrivateFrameworks/DisplayServices.framework/DisplayServices",RTLD_LAZY);
    return handle;
}
static CGDirectDisplayID selectedDisplay(NSDictionary *a) {
    if (a[@"display"]) return [a[@"display"] unsignedIntValue];
    CGDirectDisplayID ids[64]; uint32_t count = 0;
    if (CGGetOnlineDisplayList(64,ids,&count) == kCGErrorSuccess) {
        for (uint32_t i=0;i<count;i++) if (CGDisplayIsBuiltin(ids[i])) return ids[i];
    }
    return CGMainDisplayID();
}
static NSDictionary *display(NSString *op, NSDictionary *a) {
    if ([op isEqual:@"display.list"]) {
        CGDirectDisplayID ids[64]; uint32_t count=0;
        CGError e=CGGetOnlineDisplayList(64,ids,&count); if(e) return osError(@"List displays",e);
        NSMutableArray *rows=[NSMutableArray array];
        for(uint32_t i=0;i<count;i++) [rows addObject:@{@"id":@(ids[i]),@"built_in":boolean(CGDisplayIsBuiltin(ids[i])!=0),@"main":boolean(ids[i]==CGMainDisplayID()),@"width":@(CGDisplayPixelsWide(ids[i])),@"height":@(CGDisplayPixelsHigh(ids[i]))}];
        return good(rows);
    }
    CGDirectDisplayID id = a[@"id"] ? [a[@"id"] unsignedIntValue] : selectedDisplay(a);
    if (!CGDisplayIsOnline(id)) return bad(@"not_found",@"The selected display is not online.");
    if ([op isEqual:@"display.modes"] || [op isEqual:@"display.set"]) {
        CFArrayRef modes=CGDisplayCopyAllDisplayModes(id,NULL);
        if (!modes) return unsupported(@"Display modes");
        NSMutableArray *rows=[NSMutableArray array]; CGError error=kCGErrorSuccess; BOOL found=NO;
        for(CFIndex i=0;i<CFArrayGetCount(modes);i++) {
            CGDisplayModeRef mode=(CGDisplayModeRef)CFArrayGetValueAtIndex(modes,i);
            uint32_t mid=CGDisplayModeGetIODisplayModeID(mode);
            [rows addObject:@{@"id":@(mid),@"width":@(CGDisplayModeGetWidth(mode)),@"height":@(CGDisplayModeGetHeight(mode)),@"pixel_width":@(CGDisplayModeGetPixelWidth(mode)),@"pixel_height":@(CGDisplayModeGetPixelHeight(mode)),@"refresh_hz":@(CGDisplayModeGetRefreshRate(mode))}];
            if ([op isEqual:@"display.set"] && mid==[a[@"mode"] unsignedIntValue]) { found=YES; error=CGDisplaySetDisplayMode(id,mode,NULL); break; }
        }
        CFRelease(modes);
        if ([op isEqual:@"display.modes"]) return good(rows);
        if (!found) return bad(@"not_found",@"The display mode ID was not found.");
        if (error) return osError(@"Set display mode",error);
        CGDisplayModeRef active=CGDisplayCopyDisplayMode(id);
        BOOL verified=active && (uint32_t)CGDisplayModeGetIODisplayModeID(active)==[a[@"mode"] unsignedIntValue];
        if(active) CFRelease(active);
        return verified ? good(@{@"display":@(id),@"mode":a[@"mode"],@"verified":@YES}) : bad(@"verification_failed",@"The selected display mode did not become active.");
    }
    void *h=displayLibrary();
    BrightnessGet get=h ? (BrightnessGet)dlsym(h,"DisplayServicesGetBrightness") : NULL;
    BrightnessSet set=h ? (BrightnessSet)dlsym(h,"DisplayServicesSetBrightness") : NULL;
    float value=0;
    if (!get || get(id,&value)) return bad(@"unsupported",@"macOS does not expose brightness for the selected display. Use a supported built-in or Apple-controlled display; generic DDC/CI monitors are not supported.");
    NSString *action=a[@"action"] ?: @"get";
    if (![action isEqual:@"get"]) {
        float target=adjusted(a,value*100)/100.0;
        if (!set) return unsupported(@"Display brightness writing");
        int e=set(id,target); if(e) return osError(@"Set brightness",e);
        settle();
        if(get(id,&value) || fabs(value-target)>0.02) return bad(@"verification_failed",@"Display brightness did not reach the requested value.");
    }
    return good(@{@"title":@"Display",@"display":@(id),@"percent":@(value*100.0),@"backend":@"DisplayServices"});
}

@protocol MacKeyboardClient
- (NSArray *)copyKeyboardBacklightIDs;
- (BOOL)isKeyboardBuiltIn:(uint64_t)keyboard;
- (float)brightnessForKeyboard:(uint64_t)keyboard;
- (BOOL)setBrightness:(float)value fadeSpeed:(int)speed commit:(BOOL)commit forKeyboard:(uint64_t)keyboard;
@end
static NSNumber *lidClosed(void) {
    io_service_t root=IOServiceGetMatchingService(kIOMainPortDefault,IOServiceMatching("IOPMrootDomain"));
    if(!root) return nil;
    NSNumber *value=CFBridgingRelease(IORegistryEntryCreateCFProperty(root,CFSTR("AppleClamshellState"),kCFAllocatorDefault,0));
    IOObjectRelease(root);return value;
}
@protocol MacBrightnessClient
- (id)copyPropertyForKey:(NSString *)key keyboardID:(uint64_t)keyboard;
- (BOOL)setProperty:(id)value withKey:(NSString *)key keyboardID:(uint64_t)keyboard;
@end
static NSDictionary *keyboardBrightness(NSDictionary *a) {
    static void *h;
    if (!h) h=dlopen("/System/Library/PrivateFrameworks/CoreBrightness.framework/CoreBrightness",RTLD_LAZY);
    if(!h) return unsupported(@"Keyboard brightness");
    id<MacKeyboardClient> client=[NSClassFromString(@"KeyboardBrightnessClient") new];
    if (![(id)client respondsToSelector:@selector(copyKeyboardBacklightIDs)]) return unsupported(@"Keyboard brightness");
    NSArray *ids=[client copyKeyboardBacklightIDs]; if(!ids.count) return unsupported(@"A backlit keyboard");
    uint64_t keyboard=[ids[0] unsignedLongLongValue];
    for(NSNumber *n in ids) if ([(id)client respondsToSelector:@selector(isKeyboardBuiltIn:)] && [client isKeyboardBuiltIn:n.unsignedLongLongValue]) { keyboard=n.unsignedLongLongValue; break; }
    id<MacBrightnessClient> system=[NSClassFromString(@"BrightnessSystemClient") new];
    BOOL modern=[(id)system respondsToSelector:@selector(copyPropertyForKey:keyboardID:)] && [(id)system respondsToSelector:@selector(setProperty:withKey:keyboardID:)];
    NSNumber *number=modern ? [system copyPropertyForKey:@"KeyboardBacklightBrightness" keyboardID:keyboard] : nil;
    BOOL legacy=[(id)client respondsToSelector:@selector(brightnessForKeyboard:)];
    if (!number && !legacy) return unsupported(@"Keyboard brightness reading");
    BOOL useModern=number != nil;
    float value=useModern ? number.floatValue : [client brightnessForKeyboard:keyboard];
    if(!isfinite(value) || value<0 || value>1) return unsupported(@"Keyboard brightness reading");
    NSString *action=a[@"action"] ?: @"get";
    if(![action isEqual:@"get"]) {
        if(lidClosed().boolValue) return bad(@"unavailable",@"Open the MacBook lid before changing its built-in keyboard brightness.");
        float target=adjusted(a,value*100)/100.0;
        // Try the manual keyboard API first; on newer systems verify before fallback.
        BOOL wrote=NO;
        if ([(id)client respondsToSelector:@selector(setBrightness:fadeSpeed:commit:forKeyboard:)]) {
            wrote=[client setBrightness:target fadeSpeed:0 commit:YES forKeyboard:keyboard];
            settle();
            value=useModern ? [[system copyPropertyForKey:@"KeyboardBacklightBrightness" keyboardID:keyboard] floatValue] : [client brightnessForKeyboard:keyboard];
            wrote=wrote && fabs(value-target)<=0.02;
        }
        if (!wrote && useModern) {
            wrote=[system setProperty:@(target) withKey:@"KeyboardBacklightBrightness" keyboardID:keyboard];
            settle();
            number=[system copyPropertyForKey:@"KeyboardBacklightBrightness" keyboardID:keyboard];
            value=number.floatValue;
            wrote=wrote && number && fabs(value-target)<=0.02;
        }
        if(!wrote) return bad(@"verification_failed",@"Keyboard brightness did not reach the requested value.");
    }
    return good(@{@"title":@"Keyboard",@"percent":@(value*100.0),@"keyboard":@(keyboard),@"lid_closed":nilValue(lidClosed()),@"writable":boolean(!lidClosed().boolValue),@"backend":useModern ? @"CoreBrightness" : @"KeyboardBrightnessClient"});
}

static AudioObjectPropertyAddress address(AudioObjectPropertySelector selector, AudioObjectPropertyScope scope, UInt32 element) {
    return (AudioObjectPropertyAddress){selector,scope,element};
}
static BOOL readAudio(AudioObjectID device, AudioObjectPropertyAddress a, void *out, UInt32 size) { return AudioObjectGetPropertyData(device,&a,0,NULL,&size,out)==noErr; }
static BOOL writable(AudioObjectID device, AudioObjectPropertyAddress a) { Boolean yes=NO; return AudioObjectHasProperty(device,&a) && AudioObjectIsPropertySettable(device,&a,&yes)==noErr && yes; }
static NSString *audioName(AudioObjectID device) {
    CFStringRef name=NULL;
    if(!readAudio(device,address(kAudioObjectPropertyName,kAudioObjectPropertyScopeGlobal,0),&name,sizeof(name))) return @"Unknown device";
    return CFBridgingRelease(name);
}
static NSDictionary *audioOperation(NSString *op, NSDictionary *a) {
    BOOL input=[a[@"input"] boolValue];
    AudioObjectPropertyScope scope=input ? kAudioDevicePropertyScopeInput : kAudioDevicePropertyScopeOutput;
    AudioObjectPropertySelector def=input ? kAudioHardwarePropertyDefaultInputDevice : kAudioHardwarePropertyDefaultOutputDevice;
    AudioObjectPropertyAddress defaultAddress=address(def,kAudioObjectPropertyScopeGlobal,0);
    AudioDeviceID device=0;
    if(!readAudio(kAudioObjectSystemObject,defaultAddress,&device,sizeof(device)) || !device) return unsupported(@"An audio device");
    if([op isEqual:@"audio.list"]) {
        AudioObjectPropertyAddress prop=address(kAudioHardwarePropertyDevices,kAudioObjectPropertyScopeGlobal,0);
        UInt32 size=0;
        OSStatus e=AudioObjectGetPropertyDataSize(kAudioObjectSystemObject,&prop,0,NULL,&size);
        if(e) return osError(@"List audio devices",e);
        NSMutableData *data=[NSMutableData dataWithLength:size];
        e=AudioObjectGetPropertyData(kAudioObjectSystemObject,&prop,0,NULL,&size,data.mutableBytes); if(e) return osError(@"List audio devices",e);
        NSMutableArray *rows=[NSMutableArray array];
        AudioDeviceID *ids=data.mutableBytes;
        for(UInt32 i=0;i<size/sizeof(AudioDeviceID);i++) {
            UInt32 bytes=0; AudioObjectPropertyAddress streams=address(kAudioDevicePropertyStreams,scope,0);
            if(AudioObjectGetPropertyDataSize(ids[i],&streams,0,NULL,&bytes)==noErr && bytes>0) [rows addObject:@{@"id":@(ids[i]),@"name":audioName(ids[i]),@"selected":boolean(device==ids[i])}];
        }
        return good(rows);
    }
    if([op isEqual:@"audio.select"]) {
        AudioDeviceID selected=[a[@"id"] unsignedIntValue];
        UInt32 streamSize=0; AudioObjectPropertyAddress streams=address(kAudioDevicePropertyStreams,scope,0);
        if(AudioObjectGetPropertyDataSize(selected,&streams,0,NULL,&streamSize) || !streamSize) return bad(@"invalid_input",@"The device does not support the selected audio direction.");
        OSStatus e=AudioObjectSetPropertyData(kAudioObjectSystemObject,&defaultAddress,0,NULL,sizeof(selected),&selected);
        if(e) return osError(@"Select audio device",e);
        settle(); readAudio(kAudioObjectSystemObject,defaultAddress,&device,sizeof(device));
        return device==selected ? good(@{@"id":@(device),@"name":audioName(device),@"verified":@YES}) : bad(@"verification_failed",@"The default audio device did not change.");
    }
    NSString *action=a[@"action"] ?: @"get";
    AudioObjectPropertyAddress muteAddress=address(kAudioDevicePropertyMute,scope,0); UInt32 muted=0;
    BOOL hasMute=readAudio(device,muteAddress,&muted,sizeof(muted));
    if([@[@"mute",@"unmute",@"toggle"] containsObject:action]) {
        if(!hasMute || !writable(device,muteAddress)) return unsupported(@"Hardware mute");
        UInt32 target=[action isEqual:@"toggle"] ? !muted : [action isEqual:@"mute"];
        OSStatus e=AudioObjectSetPropertyData(device,&muteAddress,0,NULL,sizeof(target),&target); if(e) return osError(@"Set mute",e);
        if(!readAudio(device,muteAddress,&muted,sizeof(muted)) || muted!=target) return bad(@"verification_failed",@"The mute state did not change.");
    }
    NSMutableArray *channels=[NSMutableArray array]; Float32 sum=0;
    for(UInt32 channel=0;channel<=2;channel++) {
        Float32 v=0; AudioObjectPropertyAddress prop=address(kAudioDevicePropertyVolumeScalar,scope,channel);
        if(readAudio(device,prop,&v,sizeof(v))) { [channels addObject:@(channel)]; sum+=v; if(channel==0) break; }
    }
    BOOL levelAction=[@[@"set",@"up",@"down"] containsObject:action];
    if(levelAction) {
        if(!channels.count) return unsupported(@"Hardware volume control");
        Float32 target=adjusted(a,100*sum/channels.count)/100.0;
        for(NSNumber *ch in channels) if(!writable(device,address(kAudioDevicePropertyVolumeScalar,scope,ch.unsignedIntValue))) return unsupported(@"Hardware volume writing");
        for(NSNumber *ch in channels) {
            AudioObjectPropertyAddress prop=address(kAudioDevicePropertyVolumeScalar,scope,ch.unsignedIntValue);
            OSStatus e=AudioObjectSetPropertyData(device,&prop,0,NULL,sizeof(target),&target); if(e) return osError(@"Set volume",e);
        }
        settle(); sum=0;
        for(NSNumber *ch in channels) {
            Float32 actual=0;
            if(!readAudio(device,address(kAudioDevicePropertyVolumeScalar,scope,ch.unsignedIntValue),&actual,sizeof(actual)) || fabs(actual-target)>0.02) return bad(@"verification_failed",@"Audio volume did not reach the requested value.");
            sum+=actual;
        }
    }
    return good(@{@"title":input ? @"Input gain" : @"Volume",@"device":audioName(device),@"id":@(device),@"percent":channels.count ? @(sum/channels.count*100.0) : [NSNull null],@"muted":hasMute ? boolean(muted!=0) : [NSNull null]});
}

static NSDictionary *inputSources(NSDictionary *a) {
    NSArray *sources=CFBridgingRelease(TISCreateInputSourceList(NULL,false));
    TISInputSourceRef current=TISCopyCurrentKeyboardInputSource();
    NSString *selected=current ? (__bridge NSString *)TISGetInputSourceProperty(current,kTISPropertyInputSourceID) : nil;
    NSMutableArray *rows=[NSMutableArray array]; BOOL found=NO;
    for(id item in sources) {
        TISInputSourceRef src=(__bridge TISInputSourceRef)item;
        CFStringRef category=TISGetInputSourceProperty(src,kTISPropertyInputSourceCategory);
        if(!category || !CFEqual(category,kTISCategoryKeyboardInputSource)) continue;
        if(![(__bridge NSNumber *)TISGetInputSourceProperty(src,kTISPropertyInputSourceIsSelectCapable) boolValue]) continue;
        NSString *identifier=(__bridge NSString *)TISGetInputSourceProperty(src,kTISPropertyInputSourceID);
        NSString *name=(__bridge NSString *)TISGetInputSourceProperty(src,kTISPropertyLocalizedName);
        [rows addObject:@{@"id":nilValue(identifier),@"name":nilValue(name),@"selected":@([selected isEqual:identifier])}];
        if(a[@"id"] && [identifier isEqual:a[@"id"]]) {
            OSStatus e=TISSelectInputSource(src); if(current) CFRelease(current);
            if(e) return osError(@"Select input source",e);
            TISInputSourceRef after=TISCopyCurrentKeyboardInputSource();
            BOOL matches=after && [(__bridge NSString *)TISGetInputSourceProperty(after,kTISPropertyInputSourceID) isEqual:identifier];
            if(after) CFRelease(after);
            return matches ? good(@{@"id":identifier,@"verified":@YES}) : bad(@"verification_failed",@"The keyboard input source did not change.");
        }
    }
    if(current) CFRelease(current);
    if(a[@"id"] && !found) return bad(@"not_found",@"The keyboard input source was not found.");
    return good(rows);
}

@interface MacLocationDelegate : NSObject <CLLocationManagerDelegate>
@end
@implementation MacLocationDelegate
- (void)locationManagerDidChangeAuthorization:(CLLocationManager *)manager { (void)manager; }
@end
static NSDictionary *locationPermission(void) {
    if(!CLLocationManager.locationServicesEnabled) return bad(@"permission_required",@"Enable Location Services in System Settings > Privacy & Security to scan Wi-Fi networks.");
    CLLocationManager *manager=[CLLocationManager new];
    MacLocationDelegate *delegate=[MacLocationDelegate new]; manager.delegate=delegate;
    if(manager.authorizationStatus==kCLAuthorizationStatusNotDetermined) {
        [manager requestWhenInUseAuthorization];
        if(!pumpUntil(^BOOL{return manager.authorizationStatus!=kCLAuthorizationStatusNotDetermined;},60)) return bad(@"permission_required",@"Location permission was not granted. Run mac-cli from a local terminal and check System Settings > Privacy & Security > Location Services.");
    }
    CLAuthorizationStatus status=manager.authorizationStatus;
    if(status!=kCLAuthorizationStatusAuthorizedAlways) return bad(@"permission_required",@"Allow Location Services for the responsible terminal to access Wi-Fi network names.");
    return nil;
}
static NSDictionary *wifi(NSString *op, NSDictionary *a) {
    CWInterface *interface=[[CWWiFiClient sharedWiFiClient] interface];
    if(!interface) return unsupported(@"Wi-Fi");
    if([op isEqual:@"wifi.on"] || [op isEqual:@"wifi.off"]) {
        NSError *error=nil; BOOL target=[op hasSuffix:@".on"];
        if(![interface setPower:target error:&error]) return osError(@"Set Wi-Fi power",error.code);
        if(interface.powerOn!=target) return bad(@"verification_failed",@"Wi-Fi power did not change.");
    }
    if([op isEqual:@"wifi.scan"] || [op isEqual:@"wifi.connect"]) {
        NSDictionary *denied=locationPermission(); if(denied) return denied;
        NSError *error=nil; NSSet<CWNetwork *> *networks=[interface scanForNetworksWithName:a[@"ssid"] error:&error];
        if(!networks) return bad(@"permission_or_system_error",@"Wi-Fi scan failed. Enable Location Services for your terminal and retry.");
        NSMutableArray *rows=[NSMutableArray array];
        for(CWNetwork *network in networks) {
            if([op isEqual:@"wifi.connect"] && [network.ssid isEqual:a[@"ssid"]]) {
                if(![interface associateToNetwork:network password:a[@"password"] error:&error]) return osError(@"Connect Wi-Fi",error.code);
                return good(@{@"ssid":nilValue(interface.ssid),@"connected":@YES});
            }
            [rows addObject:@{@"ssid":nilValue(network.ssid),@"rssi":@(network.rssiValue),@"channel":@(network.wlanChannel.channelNumber)}];
        }
        if([op isEqual:@"wifi.connect"]) return bad(@"not_found",@"The network was not found. Check the SSID and Location Services permission.");
        return good(rows);
    }
    return good(@{@"enabled":@(interface.powerOn),@"interface":nilValue(interface.interfaceName),@"ssid":nilValue(interface.ssid),@"rssi":interface.ssid ? @(interface.rssiValue) : [NSNull null],@"note":interface.powerOn && !interface.ssid ? @"SSID unavailable: disconnected or Location Services access is missing." : @""});
}

@interface MacInquiry : NSObject <IOBluetoothDeviceInquiryDelegate>
@property BOOL done;
@property int error;
@end
@implementation MacInquiry
- (void)deviceInquiryComplete:(IOBluetoothDeviceInquiry *)sender error:(IOReturn)error aborted:(BOOL)aborted { self.error=error; self.done=YES; }
@end
static NSDictionary *bluetooth(NSString *op, NSDictionary *a) {
    static void *library;
    if(!library) library=dlopen("/System/Library/Frameworks/IOBluetooth.framework/IOBluetooth",RTLD_LAZY);
    int (*get)(void)=library ? dlsym(library,"IOBluetoothPreferenceGetControllerPowerState") : NULL;
    void (*set)(int)=library ? dlsym(library,"IOBluetoothPreferenceSetControllerPowerState") : NULL;
    if(!get) return unsupported(@"Bluetooth power control");
    if([op isEqual:@"bluetooth.on"] || [op isEqual:@"bluetooth.off"]) {
        if(!set) return unsupported(@"Bluetooth power writing");
        BOOL target=[op hasSuffix:@".on"]; set(target);
        if(!pumpUntil(^BOOL{ return get()==target; },3)) return bad(@"verification_failed",@"Bluetooth power did not change.");
    }
    if([op isEqual:@"bluetooth.status"] || [op hasSuffix:@".on"] || [op hasSuffix:@".off"]) return good(@{@"enabled":boolean(get()!=0)});
    if(a[@"address"]) {
        IOBluetoothDevice *device=[IOBluetoothDevice deviceWithAddressString:a[@"address"]];
        if(!device || !device.isPaired) return bad(@"not_found",@"The paired Bluetooth device was not found.");
        BOOL connect=[op hasSuffix:@".connect"];
        IOReturn e=connect ? [device openConnection] : [device closeConnection];
        if(e) return osError(@"Change Bluetooth connection",e);
        if(!pumpUntil(^BOOL{return device.isConnected==connect;},5)) return bad(@"verification_failed",@"The Bluetooth connection state did not change.");
        return good(@{@"name":nilValue(device.name),@"address":nilValue(device.addressString),@"connected":@(device.isConnected)});
    }
    NSArray *devices=[IOBluetoothDevice pairedDevices];
    if([op isEqual:@"bluetooth.scan"]) {
        MacInquiry *delegate=[MacInquiry new]; IOBluetoothDeviceInquiry *inquiry=[IOBluetoothDeviceInquiry inquiryWithDelegate:delegate];
        inquiry.inquiryLength=8;
        IOReturn e=[inquiry start]; if(e) return osError(@"Scan Bluetooth",e);
        if(!pumpUntil(^BOOL{return delegate.done;},15)) { [inquiry stop]; return bad(@"timeout",@"Bluetooth discovery timed out."); }
        if(delegate.error) return osError(@"Scan Bluetooth",delegate.error);
        devices=inquiry.foundDevices;
    }
    NSMutableArray *rows=[NSMutableArray array];
    for(IOBluetoothDevice *d in devices) [rows addObject:@{@"name":nilValue(d.name),@"address":nilValue(d.addressString),@"connected":@(d.isConnected),@"paired":@(d.isPaired)}];
    return good(rows);
}

static id plistSafe(id value) {
    if(!value) return [NSNull null];
    if([value isKindOfClass:NSDictionary.class]) {
        NSMutableDictionary *result=[NSMutableDictionary dictionary];
        for(id key in value) result[[key description]]=plistSafe(value[key]); return result;
    }
    if([value isKindOfClass:NSArray.class]) { NSMutableArray *result=[NSMutableArray array]; for(id v in value) [result addObject:plistSafe(v)]; return result; }
    if([value isKindOfClass:NSData.class]) return [value base64EncodedStringWithOptions:0];
    if([value isKindOfClass:NSDate.class]) return [(NSDate *)value description];
    return value;
}
static NSDictionary *preferences(NSString *op, NSDictionary *a) {
    CFStringRef domain=[a[@"domain"] isEqual:@"NSGlobalDomain"] ? kCFPreferencesAnyApplication : (__bridge CFStringRef)a[@"domain"];
    CFStringRef key=(__bridge CFStringRef)a[@"key"];
    if([op isEqual:@"preferences.write"]) {
        id value=a[@"value"];
        CFPreferencesSetValue(key,value==[NSNull null] ? NULL : (__bridge CFPropertyListRef)value,domain,kCFPreferencesCurrentUser,kCFPreferencesAnyHost);
        if(!CFPreferencesSynchronize(domain,kCFPreferencesCurrentUser,kCFPreferencesAnyHost)) return bad(@"write_failed",@"The preference domain could not be synchronized.");
    }
    id value=CFBridgingRelease(CFPreferencesCopyValue(key,domain,kCFPreferencesCurrentUser,kCFPreferencesAnyHost));
    return good(@{@"exists":boolean(value!=nil),@"value":plistSafe(value)});
}

static NSString *dateString(NSDate *date) { return date ? [[NSISO8601DateFormatter new] stringFromDate:date] : nil; }
static NSDate *parseDate(NSString *date) {
    NSISO8601DateFormatter *formatter=[NSISO8601DateFormatter new];
    NSDate *d=[formatter dateFromString:date];
    if(!d) { formatter.formatOptions=NSISO8601DateFormatWithInternetDateTime|NSISO8601DateFormatWithFractionalSeconds; d=[formatter dateFromString:date]; }
    return d;
}
static NSDictionary *eventKit(NSString *op, NSDictionary *a) {
    BOOL reminder=[op hasPrefix:@"reminders."];
    EKEntityType entity=reminder ? EKEntityTypeReminder : EKEntityTypeEvent;
    EKEventStore *store=[EKEventStore new];
    EKAuthorizationStatus status=[EKEventStore authorizationStatusForEntityType:entity];
    if(status!=EKAuthorizationStatusFullAccess) {
        if(status==EKAuthorizationStatusDenied || status==EKAuthorizationStatusRestricted) return bad(@"permission_required",reminder ? @"Enable Reminders access in System Settings > Privacy & Security." : @"Enable Calendars access in System Settings > Privacy & Security.");
        __block BOOL done=NO,granted=NO;
        void (^completion)(BOOL,NSError *)=^(BOOL yes,NSError *error){(void)error;dispatch_async(dispatch_get_main_queue(), ^{granted=yes;done=YES;});};
        if(reminder) [store requestFullAccessToRemindersWithCompletion:completion]; else [store requestFullAccessToEventsWithCompletion:completion];
        if(!pumpUntil(^BOOL{return done;},60)) return bad(@"timeout",@"Calendar or Reminders permission request timed out.");
        if(!granted) return bad(@"permission_required",@"Access was not granted. Check System Settings > Privacy & Security.");
    }
    if([op isEqual:@"calendar.list"] || [op isEqual:@"reminders.lists"]) {
        NSMutableArray *rows=[NSMutableArray array];
        for(EKCalendar *calendar in [store calendarsForEntityType:entity]) [rows addObject:@{@"id":calendar.calendarIdentifier,@"title":calendar.title,@"writable":@(calendar.allowsContentModifications)}];
        return good(rows);
    }
    if([op isEqual:@"calendar.add"] || [op isEqual:@"reminders.add"]) {
        NSString *cid=a[reminder ? @"list" : @"calendar"];
        EKCalendar *calendar=cid ? [store calendarWithIdentifier:cid] : (reminder ? store.defaultCalendarForNewReminders : store.defaultCalendarForNewEvents);
        if(!calendar || !calendar.allowsContentModifications || !(calendar.allowedEntityTypes & (reminder ? EKEntityMaskReminder : EKEntityMaskEvent))) return bad(@"invalid_input",@"Choose an existing writable calendar or reminder list of the correct type.");
        NSError *error=nil;
        if(reminder) {
            EKReminder *r=[EKReminder reminderWithEventStore:store]; r.title=a[@"title"]; r.calendar=calendar;
            if(![store saveReminder:r commit:YES error:&error]) return osError(@"Create reminder",error.code);
            return good(@{@"id":r.calendarItemIdentifier,@"title":r.title});
        }
        EKEvent *event=[EKEvent eventWithEventStore:store]; event.title=a[@"title"]; event.calendar=calendar; event.startDate=parseDate(a[@"start"]); event.endDate=parseDate(a[@"end"]);
        if(!event.startDate || !event.endDate || [event.startDate compare:event.endDate]!=NSOrderedAscending) return bad(@"invalid_input",@"Start and end must be RFC3339 timestamps, with end after start.");
        if(![store saveEvent:event span:EKSpanThisEvent commit:YES error:&error]) return osError(@"Create event",error.code);
        return good(@{@"id":event.eventIdentifier,@"title":event.title});
    }
    if([op hasSuffix:@".delete"] || [op isEqual:@"reminders.complete"]) {
        NSError *error=nil;
        if(reminder) {
            EKCalendarItem *item=[store calendarItemWithIdentifier:a[@"id"]];
            if(![item isKindOfClass:EKReminder.class]) return bad(@"not_found",@"The reminder ID was not found.");
            EKReminder *r=(EKReminder *)item;
            BOOL success;
            if([op hasSuffix:@".complete"]) {r.completed=YES;success=[store saveReminder:r commit:YES error:&error];}
            else success=[store removeReminder:r commit:YES error:&error];
            if(!success) return osError(@"Update reminder",error.code);
        } else {
            EKEvent *event=[store eventWithIdentifier:a[@"id"]]; if(!event) return bad(@"not_found",@"The event ID was not found.");
            if(event.hasRecurrenceRules && !a[@"start"]) return bad(@"invalid_input",@"Recurring events require --start with the occurrence start from mac calendar events.");
            if(a[@"start"]) {
                NSDate *start=parseDate(a[@"start"]);
                if(!start) return bad(@"invalid_input",@"Occurrence start must be an RFC3339 timestamp.");
                NSArray *matches=[store eventsMatchingPredicate:[store predicateForEventsWithStartDate:[start dateByAddingTimeInterval:-1] endDate:[start dateByAddingTimeInterval:1] calendars:@[event.calendar]]];
                EKEvent *occurrence=nil;
                for(EKEvent *candidate in matches) {
                    if([candidate.eventIdentifier isEqual:a[@"id"]] && fabs([candidate.startDate timeIntervalSinceDate:start])<0.5) {occurrence=candidate;break;}
                }
                if(!occurrence) return bad(@"not_found",@"The event occurrence with that ID and start time was not found.");
                event=occurrence;
            }
            if(![store removeEvent:event span:EKSpanThisEvent commit:YES error:&error]) return osError(@"Delete event",error.code);
        }
        return good(@{@"id":a[@"id"],@"completed":@YES});
    }
    NSMutableArray *rows=[NSMutableArray array];
    if(reminder) {
        NSArray *calendars=nil;
        if(a[@"list"]) { EKCalendar *c=[store calendarWithIdentifier:a[@"list"]]; if(!c) return bad(@"not_found",@"The reminder list was not found."); calendars=@[c]; }
        __block BOOL done=NO; __block NSArray *items=nil;
        [store fetchRemindersMatchingPredicate:[store predicateForRemindersInCalendars:calendars] completion:^(NSArray *found){dispatch_async(dispatch_get_main_queue(), ^{items=found;done=YES;});}];
        if(!pumpUntil(^BOOL{return done;},30)) return bad(@"timeout",@"Reading reminders timed out.");
        for(EKReminder *r in items) {
            if(a[@"query"] && [r.title rangeOfString:a[@"query"] options:NSCaseInsensitiveSearch].location==NSNotFound) continue;
            [rows addObject:@{@"id":r.calendarItemIdentifier,@"title":nilValue(r.title),@"completed":@(r.completed),@"list":r.calendar.calendarIdentifier}];
        }
    } else {
        NSDate *start,*end;
        if([op isEqual:@"calendar.today"]) { start=[[NSCalendar currentCalendar] startOfDayForDate:NSDate.date]; end=[[NSCalendar currentCalendar] dateByAddingUnit:NSCalendarUnitDay value:1 toDate:start options:0]; }
        else {start=parseDate(a[@"start"]);end=parseDate(a[@"end"]);}
        if(!start || !end || [start compare:end]!=NSOrderedAscending) return bad(@"invalid_input",@"Provide an increasing RFC3339 date range.");
        NSArray *events=[[store eventsMatchingPredicate:[store predicateForEventsWithStartDate:start endDate:end calendars:nil]] sortedArrayUsingComparator:^NSComparisonResult(EKEvent *x,EKEvent *y){return [x.startDate compare:y.startDate];}];
        for(EKEvent *event in events) [rows addObject:@{@"id":nilValue(event.eventIdentifier),@"title":nilValue(event.title),@"start":nilValue(dateString(event.startDate)),@"end":nilValue(dateString(event.endDate)),@"all_day":@(event.allDay),@"calendar":event.calendar.calendarIdentifier}];
    }
    return good(rows);
}

static NSDictionary *workspace(NSString *op, NSDictionary *a) {
    if([op isEqual:@"apps.list"]) {
        NSMutableArray *rows=[NSMutableArray array]; NSMutableSet *seen=[NSMutableSet set];
        for(NSString *root in @[@"/System/Applications",@"/Applications",[@"~/Applications" stringByExpandingTildeInPath]]) {
            NSDirectoryEnumerator *enumerator=[[NSFileManager defaultManager] enumeratorAtURL:[NSURL fileURLWithPath:root] includingPropertiesForKeys:nil options:NSDirectoryEnumerationSkipsHiddenFiles errorHandler:^BOOL(NSURL *url,NSError *error){(void)url;(void)error;return YES;}];
            for(NSURL *url in enumerator) {
                if(![url.pathExtension isEqual:@"app"]) continue;
                [enumerator skipDescendants];
                NSBundle *bundle=[NSBundle bundleWithURL:url]; NSString *identifier=bundle.bundleIdentifier;
                if(!identifier || [seen containsObject:url.path]) continue; [seen addObject:url.path];
                [rows addObject:@{@"name":[url.lastPathComponent stringByDeletingPathExtension],@"bundle_id":identifier,@"path":url.path}];
            }
        }
        return good([rows sortedArrayUsingComparator:^NSComparisonResult(NSDictionary *x,NSDictionary *y){return [x[@"name"] compare:y[@"name"]];}]);
    }
    if([op isEqual:@"apps.open"]) {
        NSString *name=a[@"name"]; NSURL *url=[[NSWorkspace sharedWorkspace] URLForApplicationWithBundleIdentifier:name];
        if(!url) {
            // Full discovery is handled in Rust; this fallback supports native names.
            for(NSString *root in @[@"/System/Applications",@"/System/Applications/Utilities",@"/Applications",[@"~/Applications" stringByExpandingTildeInPath]]) {
                NSString *path=[root stringByAppendingPathComponent:[name hasSuffix:@".app"] ? name : [name stringByAppendingString:@".app"]];
                if([[NSFileManager defaultManager] fileExistsAtPath:path]) {url=[NSURL fileURLWithPath:path];break;}
            }
        }
        if(!url) {
            NSArray *apps=workspace(@"apps.list",@{})[@"data"];
            NSMutableArray *matches=[NSMutableArray array];
            for(NSDictionary *app in apps) if([app[@"name"] caseInsensitiveCompare:name]==NSOrderedSame) [matches addObject:app];
            if(matches.count>1) return bad(@"ambiguous",@"Multiple applications match. Use a bundle ID.");
            if(matches.count==1) url=[NSURL fileURLWithPath:matches[0][@"path"]];
        }
        if(!url) return bad(@"not_found",@"The application was not found.");
        __block BOOL done=NO; __block NSError *failure=nil;
        [[NSWorkspace sharedWorkspace] openApplicationAtURL:url configuration:[NSWorkspaceOpenConfiguration configuration] completionHandler:^(NSRunningApplication *app,NSError *error){(void)app;dispatch_async(dispatch_get_main_queue(), ^{failure=error;done=YES;});}];
        if(!pumpUntil(^BOOL{return done;},30)) return bad(@"timeout",@"Opening the application timed out.");
        if(failure) return osError(@"Open application",failure.code);
        return good(@{@"opened":url.path});
    }
    if([op isEqual:@"wallpaper"]) {
        NSURL *url=[NSURL fileURLWithPath:a[@"path"]];
        if(![[NSFileManager defaultManager] fileExistsAtPath:url.path]) return bad(@"not_found",@"The wallpaper file does not exist.");
        for(NSScreen *screen in NSScreen.screens) {
            NSError *error=nil;
            if(![[NSWorkspace sharedWorkspace] setDesktopImageURL:url forScreen:screen options:@{} error:&error]) return osError(@"Set wallpaper",error.code);
        }
        return good(@{@"path":url.path});
    }
    if([op isEqual:@"clipboard.read"]) return good(@{@"text":nilValue([NSPasteboard.generalPasteboard stringForType:NSPasteboardTypeString])});
    if([op isEqual:@"clipboard.image"]) {
        NSData *png=[NSData dataWithContentsOfFile:a[@"path"]];
        if(!png || ![NSBitmapImageRep imageRepWithData:png]) return bad(@"invalid_input",@"No valid screenshot image was produced.");
        [NSPasteboard.generalPasteboard clearContents];
        if(![NSPasteboard.generalPasteboard setData:png forType:NSPasteboardTypePNG]) return bad(@"write_failed",@"The screenshot could not be copied to the clipboard.");
        return good(@{@"copied":@YES});
    }
    if([op isEqual:@"clipboard.clear"]) { [NSPasteboard.generalPasteboard clearContents]; return good(@{@"cleared":@YES}); }
    if([op isEqual:@"clipboard.write"]) {
        [NSPasteboard.generalPasteboard clearContents];
        if(![NSPasteboard.generalPasteboard setString:a[@"text"] forType:NSPasteboardTypeString]) return bad(@"write_failed",@"The clipboard could not be written.");
        return good(@{@"written":@YES});
    }
    return unsupported(@"Workspace operation");
}
static NSDictionary *ocr(NSDictionary *a) {
    VNRecognizeTextRequest *request=[VNRecognizeTextRequest new]; request.recognitionLevel=VNRequestTextRecognitionLevelAccurate;
    request.automaticallyDetectsLanguage=YES;
    NSError *error=nil;
    VNImageRequestHandler *handler=[[VNImageRequestHandler alloc] initWithURL:[NSURL fileURLWithPath:a[@"path"]] options:@{}];
    if(![handler performRequests:@[request] error:&error]) return osError(@"Recognize text",error.code);
    NSMutableArray *lines=[NSMutableArray array];
    for(VNRecognizedTextObservation *observation in request.results) {
        VNRecognizedText *candidate=[observation topCandidates:1].firstObject; if(candidate) [lines addObject:candidate.string];
    }
    return good(@{@"text":[lines componentsJoinedByString:@"\n"]});
}
static NSDictionary *localNetwork(void) {
    struct ifaddrs *head=NULL;
    if(getifaddrs(&head)) return osError(@"Read network interfaces",errno);
    NSMutableArray *rows=[NSMutableArray array];
    for(struct ifaddrs *p=head;p;p=p->ifa_next) {
        if(!p->ifa_addr || (p->ifa_addr->sa_family!=AF_INET && p->ifa_addr->sa_family!=AF_INET6)) continue;
        char host[NI_MAXHOST];
        if(!getnameinfo(p->ifa_addr,p->ifa_addr->sa_len,host,sizeof(host),NULL,0,NI_NUMERICHOST)) [rows addObject:@{@"interface":@(p->ifa_name),@"address":@(host),@"family":p->ifa_addr->sa_family==AF_INET ? @"IPv4" : @"IPv6"}];
    }
    freeifaddrs(head); return good(rows);
}
// Separate commands are intentional: a pause request must never toggle playback on.
// MediaRemote routes these to the system-selected Now Playing session.
static void *mediaLibrary(void) {
    static void *handle;
    if (!handle) handle=dlopen("/System/Library/PrivateFrameworks/MediaRemote.framework/MediaRemote",RTLD_LAZY);
    return handle;
}
static NSDictionary *mediaPlayback(NSString *op) {
    void *library=mediaLibrary();
    Boolean (*sendCommand)(unsigned int, CFDictionaryRef)=library ? dlsym(library,"MRMediaRemoteSendCommand") : NULL;
    if (!sendCommand) return unsupported(@"System media playback control");
    unsigned int command;
    NSString *action;
    if ([op isEqual:@"media.play"]) { command=0; action=@"play"; }
    else if ([op isEqual:@"media.pause"]) { command=1; action=@"pause"; }
    else return bad(@"invalid_input",@"Unsupported media action.");
    if (!sendCommand(command,NULL)) return bad(@"media_unavailable",@"macOS did not accept the media command. Start media in a compatible player and try again.");
    // Acceptance is not confirmation that an app changed state. Do not invent a
    // playback state or use a toggle fallback when no session is available.
    settle();
    return good(@{@"requested":action,@"target":@"System Now Playing session",
                  @"message":@"Media command sent. Playback depends on the active player."});
}
static NSDictionary *doctor(void) {
    BOOL screen=CGPreflightScreenCaptureAccess();
    NSOperatingSystemVersion version=NSProcessInfo.processInfo.operatingSystemVersion;
    void *lib=displayLibrary();
    NSDictionary *kbd=keyboardBrightness(@{@"action":@"get"});
    return good(@{
        @"media_control_api":@((BOOL)(mediaLibrary() && dlsym(mediaLibrary(),"MRMediaRemoteSendCommand"))),
        @"platform":@"macOS", @"architecture":@"arm64", @"version":[NSString stringWithFormat:@"%ld.%ld.%ld",(long)version.majorVersion,(long)version.minorVersion,(long)version.patchVersion],
        @"display_brightness_api":@((BOOL)(lib && dlsym(lib,"DisplayServicesGetBrightness") && dlsym(lib,"DisplayServicesSetBrightness"))),
        @"keyboard_brightness":kbd[@"error"] ? @"Unavailable" : @"Available",
        @"lid_closed":nilValue(lidClosed()),
        @"screen_recording":screen ? @"Granted" : @"Not granted; open System Settings > Privacy & Security > Screen Recording",
        @"calendar_authorization":[EKEventStore authorizationStatusForEntityType:EKEntityTypeEvent]==EKAuthorizationStatusFullAccess ? @"Granted" : @"Not granted; request with a calendar command",
        @"reminders_authorization":[EKEventStore authorizationStatusForEntityType:EKEntityTypeReminder]==EKAuthorizationStatusFullAccess ? @"Granted" : @"Not granted; request with a reminders command",
        @"location_services":@([CLLocationManager locationServicesEnabled]),
        @"note":@"Permission status checks do not request access. Hardware write support must be verified on this device."
    });
}
static NSDictionary *dispatch(NSString *op, NSDictionary *a) {
    if([op isEqual:@"battery"]) return battery();
    if([op isEqual:@"doctor"]) return doctor();
    if([op hasPrefix:@"media."]) return mediaPlayback(op);
    if([op hasPrefix:@"display."] || [op isEqual:@"brightness"]) return display(op,a);
    if([op isEqual:@"keyboard.brightness"]) return keyboardBrightness(a);
    if([op isEqual:@"keyboard.source"]) return inputSources(a);
    if([op hasPrefix:@"audio."]) return audioOperation(op,a);
    if([op hasPrefix:@"wifi."]) return wifi(op,a);
    if([op hasPrefix:@"bluetooth."]) return bluetooth(op,a);
    if([op hasPrefix:@"preferences."]) return preferences(op,a);
    if([op hasPrefix:@"calendar."] || [op hasPrefix:@"reminders."]) return eventKit(op,a);
    if([op hasPrefix:@"clipboard."] || [op hasPrefix:@"apps."] || [op isEqual:@"wallpaper"]) return workspace(op,a);
    if([op isEqual:@"session.lock"]) {
        static void *library;
        if(!library) library=dlopen("/System/Library/PrivateFrameworks/login.framework/login",RTLD_LAZY);
        int (*lockScreen)(void)=library ? dlsym(library,"SACLockScreenImmediate") : NULL;
        if(!lockScreen) return unsupported(@"Screen locking");
        int e=lockScreen(); if(e) return osError(@"Lock the screen",e);
        return good(@{@"requested":@"Lock screen"});
    }
    if([op isEqual:@"ocr"]) return ocr(a);
    if([op isEqual:@"network.ip"]) return localNetwork();
    if([op isEqual:@"plist.decode"]) {
        NSData *data=[[a[@"text"] description] dataUsingEncoding:NSUTF8StringEncoding]; NSError *error=nil;
        id value=[NSPropertyListSerialization propertyListWithData:data options:NSPropertyListImmutable format:NULL error:&error];
        return value ? good(plistSafe(value)) : osError(@"Parse property list",error.code);
    }
    return bad(@"unknown_operation",@"The native operation is not registered.");
}
char *mac_call(const char *input) {
    @autoreleasepool {
        NSDictionary *response;
        @try {
            NSData *data=[NSData dataWithBytes:input length:strlen(input)]; NSError *error=nil;
            NSDictionary *request=[NSJSONSerialization JSONObjectWithData:data options:0 error:&error];
            response=request ? dispatch(request[@"op"],request[@"args"] ?: @{}) : bad(@"invalid_input",@"The native request is invalid.");
        } @catch(NSException *exception) {
            response=bad(@"native_exception",[NSString stringWithFormat:@"The macOS operation raised %@.",exception.name]);
        }
        NSData *result=[NSJSONSerialization dataWithJSONObject:response options:NSJSONWritingFragmentsAllowed error:nil];
        if(!result) return strdup("{\"error\":{\"code\":\"native_error\",\"message\":\"The native response could not be encoded.\"}}");
        return strdup([[NSString alloc] initWithData:result encoding:NSUTF8StringEncoding].UTF8String);
    }
}
void mac_free(char *response) { free(response); }
