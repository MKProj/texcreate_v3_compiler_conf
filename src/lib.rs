use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tokio::fs::{File, read_to_string, remove_file};
use tokio::io::{AsyncWriteExt, Result};
use tokio::process::Command;
use toml::{from_str, to_string_pretty};
use termcolor::Color::Green;

macro_rules! cprint {
    ($color: expr, $($arg: tt)*) => ({
        use std::io::Write;
        use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        let _ = stdout.set_color(ColorSpec::new().set_fg(Some($color)));
        let _ = writeln!(&mut stdout, $($arg)*);
    });
}

/// The Compiler configuration allows TexCreate to compile the project
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Compiler {
    // The LaTeX compiler to use, default: pdflatex
    compiler: String,
    // The project name
    proj_name: String,
    // Any extra flags to use when compiling
    flags: Vec<String>,
    // whether to clean the out directory from `aux` and `log` files
    clean: bool,
    // whether to spawn or output the job
    mode: CompilerMode,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum CompilerMode{
    Spawn,
    Output,
}



impl Compiler {
    /// Create a new compiler configuration given a project name, and has default compiler, `pdflatex`
    pub fn new(proj_name: &str) -> Self {
        Self {
            compiler: "pdflatex".to_string(),
            proj_name: proj_name.to_string(),
            flags: vec![],
            clean: true,
            mode: CompilerMode::Output,
        }
    }
    /// Creates a `Compiler` by reading `compiler.toml`
    pub async fn from_file() -> Result<Self> {
        let s = read_to_string("compiler.toml").await?;
        Ok(from_str(&s).unwrap())
    }
    /// Turns `Compiler` into a TOML string
    pub fn to_string(&self) -> String {
        to_string_pretty(&self).unwrap()
    }
    /// Creates a new `compiler.toml` file.
    ///
    /// Since `Compiler` contains the field, `proj_name`, the file will be created
    /// in the correct path.
    pub async fn create_file(&self) -> Result<()> {
        let s = self.to_string();
        let path = PathBuf::from(&self.proj_name).join("compiler.toml");
        let mut file = File::create(path).await?;
        file.write_all(s.as_bytes()).await?;
        Ok(())
    }

    async fn output(&self){
        let _ = Command::new(&self.compiler)
            .arg("-output-directory=out")
            .args(&self.flags)
            .arg(&self.proj_name)
            .output()
            .await
            .expect("Couldn't compile LaTeX document");
    }

    async fn spawn(&self){
        let _ = Command::new(&self.compiler)
            .arg("-output-directory=out")
            .args(&self.flags)
            .arg(&self.proj_name)
            .spawn()
            .expect("Compiler failed to start")
            .wait()
            .await
            .expect("Couldn't compile LaTeX document");
    }

    /// Compiles a TexCreate project
    ///
    /// The following command is used:
    /// ```bash
    /// # using pdflatex as example compiler
    /// $ pdflatex -output-directory=out <flags> `proj_name`.tex
    /// ```
    pub async fn compile(&self) -> Result<()> {
        // run the compile command
        match self.mode{
            CompilerMode::Spawn => self.spawn().await,
            CompilerMode::Output => self.output().await
        }
        if self.clean{
            // clean the out directory by removing the aux and log files
            // should exist if the project compiled successfully
            let out = PathBuf::from("out");
            let aux = out.join(format!("{}.aux", &self.proj_name));
            let log = out.join(format!("{}.log", &self.proj_name));
            remove_file(aux).await?;
            remove_file(log).await?;
        }
        // if nothing panicked then we have a successful compile
        cprint!(Green, "The project `{}` successfully compiled!", &self.proj_name);
        Ok(())
    }
}

