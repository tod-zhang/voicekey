use serde::{Deserialize, Serialize};
use voicekey_core::util::data_file;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    #[default]
    Doubao,
    Wetype,
    Qwen,
    Baidu,
    Sogou,
    Iflytek,
    Offline,
    WetypeOffline,
    All,
}

impl Channel {
    pub fn title(self) -> &'static str {
        match self {
            Channel::Doubao => "豆包输入法",
            Channel::Wetype => "微信输入法",
            Channel::Qwen => "千问输入法",
            Channel::Baidu => "百度输入法",
            Channel::Sogou => "搜狗输入法",
            Channel::Iflytek => "讯飞输入法",
            Channel::Offline => "豆包离线",
            Channel::WetypeOffline => "微信离线",
            Channel::All => "多渠道（说完挑选）",
        }
    }

    /// 当前平台可用的识别渠道（不含「全部」）
    pub fn engines() -> Vec<Channel> {
        let mut v = vec![Channel::Doubao, Channel::Wetype, Channel::Qwen, Channel::Baidu, Channel::Sogou, Channel::Iflytek];
        if crate::offline::SUPPORTED {
            v.push(Channel::Offline);
        }
        v.push(Channel::WetypeOffline);
        v
    }

    pub fn available() -> Vec<Channel> {
        let mut v = Self::engines();
        v.push(Channel::All);
        v
    }
}

/// 点按快捷键：单个修饰键（区分左右）或 修饰键+按键 组合
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut {
    pub code: u32,
    pub mods: u8,
    pub name: String,
}

impl Shortcut {
    pub fn is_modifier(&self) -> bool {
        voicekey_platform::modifier(self.code).is_some()
    }

    pub fn title(&self) -> String {
        voicekey_platform::mods_prefix(self.mods) + &self.name
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub channel: Channel,
    pub hold_key: String,
    pub tap_shortcut: Option<Shortcut>,
    pub silence: f64,
    pub streaming: bool,
    pub live_text: bool,
    pub mic: String,
    /// 多渠道模式下同时识别的渠道（至少 2 个）
    pub multi: Vec<Channel>,
    /// 候选框上次选中的渠道，下次默认聚焦
    pub last_pick: Option<Channel>,
    pub autostart: bool,
    pub theme: String,
    pub onboarded: bool,
    /// 远程桌面兼容（Windows）：前台是这些程序时改为等剪贴板同步后粘贴
    pub remote_compat: bool,
    /// 进程名包含其中任一项（不区分大小写）即视为远程桌面客户端
    pub remote_apps: Vec<String>,
    /// 写入剪贴板后等多久再按粘贴键，给远程软件同步剪贴板留时间
    pub remote_delay: u64,
}

impl Default for Settings {
    fn default() -> Self {
        let (hold, tap) = if cfg!(target_os = "macos") { ("rightAlt", (54, "右 ⌘")) } else { ("rightAlt", (0xA3, "右 Ctrl")) };
        Self {
            channel: Channel::Doubao,
            hold_key: hold.into(),
            tap_shortcut: Some(Shortcut { code: tap.0, mods: 0, name: tap.1.into() }),
            silence: 1.5,
            streaming: true,
            live_text: true,
            mic: String::new(),
            multi: Channel::engines().into_iter().filter(|&c| c != Channel::WetypeOffline).collect(),
            last_pick: None,
            autostart: false,
            theme: "system".into(),
            onboarded: false,
            remote_compat: true,
            remote_apps: ["uu", "todesk", "sunlogin", "anydesk", "rustdesk", "parsec"].map(String::from).to_vec(),
            remote_delay: 400,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let mut s: Settings = std::fs::read(data_file("settings.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        s.sanitize();
        s
    }

    pub fn save(&self) {
        if let Ok(b) = serde_json::to_vec_pretty(self) {
            let _ = std::fs::write(data_file("settings.json"), b);
        }
    }

    /// 不可用的渠道/按键回落默认
    pub fn sanitize(&mut self) {
        if !Channel::available().contains(&self.channel) {
            self.channel = Channel::Doubao;
        }
        if !voicekey_platform::hold_keys().iter().any(|k| k.id == self.hold_key) {
            self.hold_key = voicekey_platform::hold_keys()[0].id.into();
        }
        self.silence = self.silence.clamp(1.0, 5.0);
        self.remote_delay = self.remote_delay.clamp(100, 2000);
        let engines = Channel::engines();
        let mut multi: Vec<Channel> = engines.iter().copied().filter(|c| self.multi.contains(c)).collect();
        if multi.len() < 2 {
            multi = engines;
        }
        self.multi = multi;
    }

    /// 输出目标是否为远程桌面客户端
    pub fn is_remote(&self, exe: &str) -> bool {
        self.remote_compat
            && self.remote_apps.iter().map(|a| a.trim().to_lowercase()).any(|a| !a.is_empty() && exe.contains(&a))
    }

    pub fn hold_code(&self) -> u32 {
        voicekey_platform::hold_keys().iter().find(|k| k.id == self.hold_key).map_or(0, |k| k.code)
    }

    pub fn hold_name(&self) -> &'static str {
        voicekey_platform::hold_keys().iter().find(|k| k.id == self.hold_key).map_or("", |k| k.name)
    }
}
