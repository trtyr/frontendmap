use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Frontend project map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendMap {
    pub schema_version: u32,
    pub project: ProjectInfo,
    pub components: Vec<Component>,
    pub routes: Vec<Route>,
    pub api_calls: Vec<ApiCall>,
    pub stores: Vec<Store>,
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub root: PathBuf,
    pub name: String,
    pub framework: Framework,
    pub features: Vec<String>,
    pub tech_stack: Vec<String>,
    pub file_count: usize,
    pub component_count: usize,
}

/// Frontend framework
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Framework {
    React,
    Vue,
    Svelte,
    Angular,
    Next,
    Nuxt,
    SvelteKit,
    Solid,
    Preact,
    Qwik,
    Astro,
    Unknown,
}

impl Framework {
    pub fn as_str(&self) -> &'static str {
        match self {
            Framework::React => "react",
            Framework::Vue => "vue",
            Framework::Svelte => "svelte",
            Framework::Angular => "angular",
            Framework::Next => "next",
            Framework::Nuxt => "nuxt",
            Framework::SvelteKit => "sveltekit",
            Framework::Solid => "solid",
            Framework::Preact => "preact",
            Framework::Qwik => "qwik",
            Framework::Astro => "astro",
            Framework::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for Framework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// UI Component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub file: PathBuf,
    pub kind: ComponentKind,
    pub props: Vec<Prop>,
    pub used_by: Vec<PathBuf>,
    pub uses: Vec<String>,
    pub line: usize,
}

/// Component type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComponentKind {
    Function,
    Class,
    Arrow,
}

impl std::fmt::Display for ComponentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentKind::Function => write!(f, "function"),
            ComponentKind::Class => write!(f, "class"),
            ComponentKind::Arrow => write!(f, "arrow"),
        }
    }
}

/// Component prop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prop {
    pub name: String,
    pub type_annotation: Option<String>,
    pub required: bool,
}

/// Route definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub path: String,
    pub component: String,
    pub file: PathBuf,
    pub line: usize,
}

/// API call site
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCall {
    pub component: String,
    pub file: PathBuf,
    pub endpoint: String,
    pub method: String,
    pub line: usize,
}

/// State store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    pub name: String,
    pub file: PathBuf,
    pub kind: StoreKind,
    pub subscribers: Vec<String>,
}

/// Store type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StoreKind {
    Redux,
    Zustand,
    Pinia,
    Vuex,
    Context,
    Jotai,
    Recoil,
    Mobx,
    Valtio,
    Xstate,
    Nanostores,
    Unknown,
}

impl StoreKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            StoreKind::Redux => "redux",
            StoreKind::Zustand => "zustand",
            StoreKind::Pinia => "pinia",
            StoreKind::Vuex => "vuex",
            StoreKind::Context => "context",
            StoreKind::Jotai => "jotai",
            StoreKind::Recoil => "recoil",
            StoreKind::Mobx => "mobx",
            StoreKind::Valtio => "valtio",
            StoreKind::Xstate => "xstate",
            StoreKind::Nanostores => "nanostores",
            StoreKind::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for StoreKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
