use std::{env, io::BufRead, process::Command, str::from_utf8};

use serde_json::{Value, json};

fn main() {
    // cargo rxmkfs <mkfs> progs -- rustc parameters
    // [0] --> cargo-rxmkfs
    // [1] --> rxmkfs
    // [2] --> mkfs
    // [3] --> output file

    // 0. 命令行参数处理以及确定工具链
    let cargo = env::var("CARGO").unwrap_or("cargo".to_owned());
    // 其余配置都在环境变量中 and inherited
    let mkfs = env::args().nth(2).unwrap();
    let outfile = env::args().nth(3).unwrap();
    let mut uprogs: Vec<_> = env::args().skip(4).take_while(|arg| arg != "--").collect();
    uprogs.pop();

    let compile_opts: Vec<_> = env::args().skip_while(|s| s != "--").skip(1).collect();

    // 1. 编译当前指定的rust bin creates
    let mut cmd = Command::new(cargo);
    cmd.arg("rustc").arg("--message-format").arg("json");
    cmd.args(compile_opts);
    let compile_output = cmd.output().unwrap();
    // 收集所有的Executable
    let mut executables = vec![];
    for line in String::from_utf8(compile_output.stdout).unwrap().lines() {
        let line_json: Value = serde_json::from_str(line).unwrap();
        if let Value::String(exe) = &line_json["executable"] {
            executables.push(exe.clone());
        }
    }

    // 2. 收集其它需要构建的文件 -- uprogs

    // 3. 用传入的mkfs程序构建fs.img
    let mut cmd = Command::new(mkfs);
    cmd.arg(outfile);
    cmd.args(uprogs).args(executables);
    println!("Build fs.img: {:?}", cmd.output().unwrap());
}
