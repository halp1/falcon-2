macro_rules! const_for_dynamic {
  ($name:ident in 0..$limit:expr => $body:block) => {{
    macro_rules! exec_step {
      ($val:expr) => {{
        const $name: usize = $val;
        if $name < const { $limit } {
          $body
        }
      }};
    }

    exec_step!(0);
    exec_step!(1);
    exec_step!(2);
    exec_step!(3);
    exec_step!(4);
    exec_step!(5);
    exec_step!(6);
    exec_step!(7);
    exec_step!(8);
    exec_step!(9);
    exec_step!(10);
    exec_step!(11);
    exec_step!(12);
    exec_step!(13);
    exec_step!(14);
    exec_step!(15);
  }};
}

pub(crate) use const_for_dynamic;
