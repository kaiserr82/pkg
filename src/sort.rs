use crate::manager::manager_priority;
use crate::ranking::{classify_package, language_bonus, type_priority};
use crate::language::system_language;
use crate::search::UnifiedPackage;

pub fn sorter_priority(mut result: Vec<UnifiedPackage>) -> Vec<UnifiedPackage> {
    let sys_lang = system_language();

    result.sort_by(|a, b| {
        let a_type = classify_package(&a.name);
        let b_type = classify_package(&b.name);

        let a_lang = language_bonus(&a.name, &sys_lang);
        let b_lang = language_bonus(&b.name, &sys_lang);

        let a_mgr = manager_priority(&a.manager);
        let b_mgr = manager_priority(&b.manager);

        let a_score = a.score + a_lang;
        let b_score = b.score + b_lang;

        (
            a_score,
            a_mgr,
            type_priority(&a_type),
        )
        .cmp(&(
            b_score,
            b_mgr,
            type_priority(&b_type),
        ))
        .reverse()
    });

    result
}