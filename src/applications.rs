use crate::{
    error::{Error, Result},
    native, process,
};
use serde_json::{json, Value};
use std::time::Duration;

pub const ALIASES: &[(&str, &str)] = &[
    ("weather", "com.apple.weather"),
    ("activity-monitor", "com.apple.ActivityMonitor"),
    ("disk-utility", "com.apple.DiskUtility"),
    ("system-information", "com.apple.SystemProfiler"),
    ("console", "com.apple.Console"),
    ("terminal", "com.apple.Terminal"),
    ("screenshot", "com.apple.screencaptureui"),
    ("audio-midi-setup", "com.apple.audio.AudioMIDISetup"),
    ("calendar", "com.apple.iCal"),
    ("reminders", "com.apple.reminders"),
    ("notes", "com.apple.Notes"),
    ("music", "com.apple.Music"),
    ("shortcuts", "com.apple.shortcuts"),
    ("clock", "com.apple.clock"),
    ("settings", "com.apple.systempreferences"),
    ("finder", "com.apple.finder"),
];
pub fn open(name: &str) -> Result<Value> {
    let name = ALIASES
        .iter()
        .find(|(alias, _)| *alias == name)
        .map(|(_, id)| *id)
        .unwrap_or(name);
    native::call("apps.open", json!({"name":name}))
}
pub fn list() -> Result<Value> {
    Ok(
        json!({"aliases":ALIASES.iter().map(|(alias,id)|json!({"alias":alias,"bundle_id":id})).collect::<Vec<_>>(),"applications":native::call("apps.list",json!({}))?}),
    )
}
pub fn jxa(script: &str, args: &[String]) -> Result<Value> {
    let mut arguments = vec![
        "-l".into(),
        "JavaScript".into(),
        "-e".into(),
        script.into(),
        "--".into(),
    ];
    arguments.extend_from_slice(args);
    let output=process::run("/usr/bin/osascript",&arguments,None,Duration::from_secs(90)).map_err(|e|{
        if e.message.contains("-1743"){Error::new("permission_required","Allow Automation access to this application in System Settings > Privacy & Security > Automation.")}
        else if e.message.contains("-1728"){Error::new("not_found","The requested application item was not found.")}
        else{e}
    })?;
    Ok(serde_json::from_str(output.trim())?)
}
const NOTES: &str = r#"
function run(argv) {
    const app=Application('com.apple.Notes'), action=argv[0];
    const summarize=n=>({id:n.id(),title:n.name()});
    if(action==='folders') return JSON.stringify(app.folders().map(f=>({id:f.id(),name:f.name()})));
    if(action==='list') return JSON.stringify(app.notes().map(summarize));
    if(action==='search') return JSON.stringify(app.notes().filter(n=>n.name().toLowerCase().includes(argv[1].toLowerCase())).map(summarize));
    if(action==='read') { const n=app.notes.byId(argv[1]); return JSON.stringify({id:n.id(),title:n.name(),text:n.plaintext()}); }
    if(action==='add') {
        const escape=s=>s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;').replace(/\n/g,'<br>');
        const folder=argv[3] ? app.folders.byId(argv[3]) : app.defaultAccount().defaultFolder();
        const note=app.Note({body:'<h1>'+escape(argv[1])+'</h1><div>'+escape(argv[2])+'</div>'});
        folder.notes.push(note);
        return JSON.stringify({created:true,title:argv[1]});
    }
    throw Error('Unknown Notes operation.');
}
"#;
pub fn notes(action: &str, args: &[String]) -> Result<Value> {
    let mut all = vec![action.into()];
    all.extend_from_slice(args);
    jxa(NOTES, &all)
}
const MUSIC: &str = r#"
function run(argv) {
    const app=Application('com.apple.Music'), action=argv[0];
    if(action==='status' && !app.running()) return JSON.stringify({running:false,state:'Not running'});
    if(action==='play') app.play();
    if(action==='pause') app.pause();
    if(action==='toggle') app.playpause();
    if(action==='next') app.nextTrack();
    if(action==='previous') app.previousTrack();
    const state=app.playerState();
    if(state==='stopped') return JSON.stringify({state:state});
    const track=app.currentTrack();
    return JSON.stringify({state:state,title:track.name(),artist:track.artist(),album:track.album(),position_seconds:app.playerPosition(),duration_seconds:track.duration()});
}
"#;
pub fn music(action: &str) -> Result<Value> {
    jxa(MUSIC, &[action.into()])
}
pub fn appearance(mode: Option<&str>) -> Result<Value> {
    let script = r#"
function run(argv) {
    const prefs=Application('System Events').appearancePreferences;
    if(argv.length) prefs.darkMode=argv[0]==='dark';
    return JSON.stringify({mode:prefs.darkMode() ? 'dark':'light'});
}
"#;
    jxa(
        script,
        &mode.into_iter().map(String::from).collect::<Vec<_>>(),
    )
}
pub fn empty_trash() -> Result<Value> {
    jxa("function run(){Application('com.apple.finder').emptyTrash();return JSON.stringify({emptied:true});}",&[])
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn utility_aliases_are_unique() {
        let ids: std::collections::BTreeSet<_> = ALIASES.iter().map(|(a, _)| a).collect();
        assert_eq!(ids.len(), ALIASES.len());
        assert!(ALIASES.iter().any(|(a, _)| *a == "weather"));
    }
}
