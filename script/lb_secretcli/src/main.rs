mod lb_factory;
mod lb_pair;

pub fn main() -> std::io::Result<()> {
    lb_factory::main()?;
    lb_pair::main()?;

    Ok(())
}
