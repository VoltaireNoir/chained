use chained::*;

#[test]
fn lazy_eval() {
    let lazy = chained!(0, |x| x + 100, |x| x + 100);
    let lazy = lazy.chain(|x| x / 2);
    assert_eq!(100, lazy.eval())
}

#[test]
fn bare_bones() {
    assert_eq!(Link::new(180).chain(|x| x + x).eval(), 360)
}

#[test]
fn accross_threads() {
    let lazy = chained!("\n", |x| x.trim(), |x| x.len());
    let h = std::thread::spawn(|| lazy.eval());
    assert_eq!(h.join().unwrap(), 0)
}

#[test]
fn cloned() {
    let x = chained!(0..100, |x| x.sum::<usize>());
    let y = x.clone();
    assert_eq!(x.eval(), y.eval())
}

mod inter_chain {
    use std::{
        ops::{Deref, DerefMut},
        path::Path,
    };

    use chained::*;

    #[test]
    fn owned() {
        let chain = "test".into_chained(|x| x.len()).eval();
        assert_eq!(4, chain)
    }

    #[test]
    fn clone() {
        #[derive(Clone, PartialEq, Debug)]
        struct Test;
        let x = Test;
        let y = x.to_chained(|x| x);

        assert_eq!(x, y.eval())
    }

    #[test]
    fn shared_ref() {
        let x = String::from("hello");
        x.chained(|x| x.bytes().collect::<Vec<u8>>()).eval();
        assert_eq!(x, String::from("hello"))
    }

    #[test]
    fn mut_ref() {
        let mut st = "test".to_owned();
        st.chained_mut(|x| x.push('.')).eval();
        assert_eq!(st, String::from("test."))
    }

    struct Test(String);
    impl Deref for Test {
        type Target = String;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl DerefMut for Test {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    #[test]
    fn deref() {
        let t = Test(String::from(""));
        let x = t.chained_deref(|x| assert_eq!(x, ""));
        x.eval();
    }

    #[test]
    fn deref_mut() {
        let mut t = Test(String::new());
        t.chained_deref_mut(|x| x.push_str("hello")).eval();
        assert_eq!(*t, "hello")
    }

    #[test]
    fn as_ref() {
        let takes_path = |x: &Path| x.is_dir();
        let r = "abcdefghijkl".chained_as_ref(takes_path).eval();
        assert!(!r)
    }

    #[test]
    fn as_mut() {
        let mut b: Box<String> = "abc".to_owned().into();
        let takes_ms = |x: &mut String| x.clear();
        b.chained_as_mut(takes_ms).eval();
        assert_eq!(*b, "")
    }
}

mod standalone_macro {
    use chained::chained;

    #[test]
    fn standalone_macro_eager() {
        let x = chained!(>> 1, |x| x + 1);
        assert_eq!(x, 2);
    }

    #[test]
    fn standalone_macro_lazy() {
        let x = chained!(1, |x| x + 1);
        let y = chained!(>>> x, |y| y * 2);
        assert_eq!(y, 4);
    }
}
