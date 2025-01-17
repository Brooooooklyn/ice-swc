use crate::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use swc_common::DUMMY_SP;
use swc_ecmascript::ast::{
    BindingIdent, Bool, Decl, Expr, Ident, ImportNamedSpecifier, ImportSpecifier, Lit, ModuleDecl,
    ModuleItem, Pat, Stmt, VarDecl, VarDeclKind, VarDeclarator,
};

use swc_ecmascript::visit::Fold;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct KeepPlatformPatcher {
    pub platform: String,
}

/// Configuration related to source map generated by swc.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum KeepPlatformConfig {
    Bool(bool),
    KeepPlatform(String),
}

impl Default for KeepPlatformConfig {
    fn default() -> Self {
        KeepPlatformConfig::Bool(false)
    }
}

pub fn keep_platform(options: KeepPlatformConfig) -> impl Fold {
    let platform: String = match options {
        KeepPlatformConfig::KeepPlatform(platform) => platform,
        _ => "".to_string(),
    };
    KeepPlatformPatcher { platform: platform }
}

// platform maps
lazy_static! {
    static ref PLATFORM_MAP: HashMap<String, Vec<String>> = HashMap::from([
        ("web".to_string(), vec!["isWeb".to_string()]),
        ("node".to_string(), vec!["isNode".to_string()]),
        ("weex".to_string(), vec!["isWeex".to_string()]),
        (
            "kraken".to_string(),
            vec!["isKraken".to_string(), "isWeb".to_string()]
        ),
        (
            "wechat-miniprogram".to_string(),
            vec![
                "isWeChatMiniProgram".to_string(),
                "isWeChatMiniprogram".to_string()
            ]
        ),
        ("miniapp".to_string(), vec!["isMiniApp".to_string()]),
        (
            "bytedance-microapp".to_string(),
            vec!["isByteDanceMicroApp".to_string()]
        ),
        (
            "kuaishou-miniprogram".to_string(),
            vec!["isKuaiShouMiniProgram".to_string()]
        ),
        (
            "baidu-smartprogram".to_string(),
            vec!["isBaiduSmartProgram".to_string()]
        ),
    ]);
}

impl Fold for KeepPlatformPatcher {
    fn fold_module_items(&mut self, items: Vec<ModuleItem>) -> Vec<ModuleItem> {
        // Get platform flag, such as ["isWeb"]
        let platform_flags: Vec<String> = match PLATFORM_MAP.get(&self.platform.to_string()) {
            Some(flags) => flags.to_vec(),
            None => vec![],
        };
        // Collect top-level expression
        let mut new_module_items: Vec<ModuleItem> = vec![];
        // Save isWeb/isWeex into env_variables
        let mut env_variables: Vec<&Ident> = vec![];
        for module_item in items.iter() {
            match module_item {
                ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) => {
                    if &import_decl.src.value == "universal-env" {
                        for specifier in import_decl.specifiers.iter() {
                            match specifier {
                                ImportSpecifier::Named(named) => {
                                    let ImportNamedSpecifier {
                                        local,
                                        span: _,
                                        imported: _,
                                        is_type_only: _,
                                    } = named;
                                    env_variables.push(local);
                                }
                                _ => {}
                            }
                        }
                    } else {
                        new_module_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(
                            import_decl.clone(),
                        )))
                    }
                }
                _ => new_module_items.push(module_item.clone()),
            }
        }

        // If it exist env variables, we need insert declare expression
        if env_variables.len() > 0 {
            for env_variable in env_variables {
                let decs: Vec<VarDeclarator> = vec![VarDeclarator {
                    span: DUMMY_SP,
                    definite: false,
                    name: Pat::Ident(BindingIdent {
                        id: env_variable.clone(),
                        type_ann: Default::default(),
                    }),
                    // Init value, such as var isWeb = true
                    init: Option::Some(Box::new(Expr::Lit(Lit::Bool(Bool {
                        value: platform_flags.contains(&env_variable.sym.to_string()),
                        span: Default::default(),
                    })))),
                }];
                new_module_items.insert(
                    0,
                    ModuleItem::Stmt(Stmt::Decl(Decl::Var(VarDecl {
                        span: DUMMY_SP,
                        kind: VarDeclKind::Var,
                        declare: false,
                        decls: decs,
                    }))),
                );
            }
        }
        return new_module_items;
    }
}
