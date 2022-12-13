#![allow(clippy::new_without_default)]
pub mod exec;
pub mod model;

// fn _main() {
//     let instrs = vec![
//         Instr::NumPush(34),
//         Instr::NumPush(34),
//         Instr::FuncMake(
//             vec!["abc".to_string()],
//             vec![
//                 Instr::Get("abc".to_string()),
//                 Instr::NumPush(1),
//                 Instr::NumAdd,
//             ],
//         ),
//         Instr::FuncApply,
//         Instr::NumAdd,
//         Instr::Print,
//     ];

//     // instrs.iter().for_each(|instr| {
//     //     println!(
//     //         "{}",
//     //         instr
//     //             .to_bytes()
//     //             .iter()
//     //             .map(|b| format!("{:02x}", b))
//     //             .collect::<Vec<_>>()
//     //             .join(" ")
//     //     )
//     // });

//     let mut executor = Executor::new(instrs);
//     match executor.run() {
//         Ok(()) => (),
//         Err(e) => println!("{:?}", e),
//     }
// }
