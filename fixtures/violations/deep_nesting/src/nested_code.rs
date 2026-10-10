//! Negative fixture: Control-flow nesting depth exceeding 4 (reaches depth 5).

pub fn deeply_nested_logic(flag1: bool, flag2: bool, flag3: bool, flag4: bool, flag5: bool) -> i32 {
    let mut result = 0;
    if flag1 {
        if flag2 {
            if flag3 {
                if flag4 {
                    if flag5 {
                        result = 42;
                    }
                }
            }
        }
    }
    result
}
