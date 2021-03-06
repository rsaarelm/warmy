//! Module exporting all key types recognized by `warmy`.

use any_cache::CacheKey;
use std::hash::{Hash, Hasher};
use std::fmt::{self, Display};
use std::marker::PhantomData;
use std::path::{Component, Path, PathBuf};

use res::Res;

/// Class of keys recognized by `warmy`.
pub trait Key: 'static + Clone + Eq + Hash {
  /// Prepare a key.
  ///
  /// If your key is akin to a file system key, it’s very likely you need to substitute its VFS path
  /// with the `root` argument. It’s advised to use the `prepare_key` method for your inner key value.
  fn prepare_key(self, root: &Path) -> Self;
}

/// A key that can either be a path or a logical location.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SimpleKey {
  /// A key to a resource living on the filesystem.
  Path(PathBuf),
  /// A key to a resource living in memory or computed on the fly.
  Logical(String),
}

impl SimpleKey {
  pub fn from_path<P>(path: P) -> Self where P: AsRef<Path> {
    SimpleKey::Path(path.as_ref().to_owned())
  }
}

impl<'a>  From<&'a Path> for SimpleKey {
  fn from(path: &Path) -> Self {
    SimpleKey::from_path(path)
  }
}
impl From<PathBuf> for SimpleKey {
  fn from(path: PathBuf) -> Self {
    SimpleKey::Path(path)
  }
}

impl<'a> From<&'a str> for SimpleKey {
  fn from(s: &str) -> Self {
    SimpleKey::Logical(s.to_owned())
  }
}

impl From<String> for SimpleKey {
  fn from(s: String) -> Self {
    SimpleKey::Logical(s)
  }
}

impl Display for SimpleKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      SimpleKey::Path(ref path) => write!(f, "{}", path.display()),
      SimpleKey::Logical(ref name) => write!(f, "{}", name)
    }
  }
}

impl Key for SimpleKey {
  fn prepare_key(self, root: &Path) -> Self {
    match self {
      SimpleKey::Path(path) => SimpleKey::Path(vfs_substite_path(&path, root)),
      SimpleKey::Logical(x) => SimpleKey::Logical(x),
    }
  }
}
/// Substitute a VFS path by a real one.
fn vfs_substite_path(path: &Path, root: &Path) -> PathBuf {
  let mut components = path.components().peekable();
  let root_components = root.components();

  match components.peek() {
    Some(&Component::RootDir) => {
      // drop the root component
      root_components.chain(components.skip(1)).collect()
    }

    _ => root_components.chain(components).collect(),
  }
}

pub(crate) struct PrivateKey<K, T>(pub(crate) K, PhantomData<T>);

impl<K, T> PrivateKey<K, T> {
  pub(crate) fn new(key: K) -> Self {
    PrivateKey(key, PhantomData)
  }
}

impl<K, T> Hash for PrivateKey<K, T> where K: Hash {
  fn hash<H>(&self, state: &mut H) where H: Hasher {
    self.0.hash(state)
  }
}

impl<K, T> CacheKey for PrivateKey<K, T> where T: 'static, K: Key {
  type Target = Res<T>;
}
