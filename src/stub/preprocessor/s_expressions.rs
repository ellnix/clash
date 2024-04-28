use std::cell::RefCell;

use itertools::Itertools;

use super::Renderable;
use crate::stub::{Cmd, Stub, VariableCommand};

pub fn transform(stub: &mut Stub) {
    let old_commands = stub.commands.drain(..).peekable();

    stub.commands = old_commands
        .batching(|it| {
            let cmd = it.next()?;
            if let Cmd::Read(vars) = cmd {
                let mut reads: Vec<Vec<VariableCommand>> = vec![vars];

                while let Some(Cmd::Read(vars)) = it.next_if(|cmd| matches!(cmd, Cmd::Read(_))) {
                    reads.push(vars);
                }

                let batch = ReadBatch::new(reads, RefCell::new(Vec::new()));
                Some(Cmd::External(Box::new(batch)))
            } else {
                Some(cmd)
            }
        })
        .collect();

    let read_batches: Vec<(usize, &ReadBatch)> = stub
        .commands
        .iter()
        .enumerate()
        .filter_map(|(i, cmd)| {
            if let Cmd::External(renderable) = cmd {
                Some((i, renderable.as_any().downcast_ref::<ReadBatch>()?))
            } else {
                None
            }
        })
        .collect();

    for (i, read_batch) in read_batches {
        let mut nested_cmds: Vec<&Cmd> = stub.commands[i..].iter().collect();
        read_batch.nested_cmds.borrow_mut().append(&mut nested_cmds);
    }
}

#[derive(Debug, Clone)]
struct ReadBatch<'a> {
    pub read_lines: Vec<Vec<VariableCommand>>,
    pub nested_cmds: RefCell<Vec<&'a Cmd<'a>>>,
}

impl<'a> ReadBatch<'a> {
    fn new(read_lines: Vec<Vec<VariableCommand>>, nested_cmds: RefCell<Vec<&'a Cmd<'a>>>) -> ReadBatch<'a> {
        ReadBatch {
            read_lines,
            nested_cmds,
        }
    }
}

impl<'a> Renderable<'a> for ReadBatch<'a> {
    fn render(&self, renderer: &crate::stub::renderer::Renderer) -> String {
        let nested_string: String =
            self.nested_cmds.borrow().iter().map(|cmd| renderer.render_command(cmd, 0)).collect();
        let nested_lines: Vec<&str> = nested_string.lines().collect();
        let mut context = tera::Context::new();
        context.insert("read_lines", &self.read_lines);
        context.insert("nested_lines", &nested_lines);
        renderer.tera_render("read_batch", &mut context)
    }

    fn as_any(&self) -> &(dyn std::any::Any + 'a) {
        self
    }
}
