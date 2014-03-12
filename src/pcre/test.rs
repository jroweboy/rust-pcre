extern crate pcre;
extern crate collections;

use collections::EnumSet;
use pcre::{CompileOption, StudyOption, ExtraOption, Pcre};

#[test]
#[should_fail]
fn test_compile_nul() {
    // Nul bytes are not allowed in the pattern string.
    drop(Pcre::compile("\0abc"));
}

#[test]
fn test_compile_bad_pattern() {
    let err = Pcre::compile("[").unwrap_err();
    assert_eq!(err.offset(), 1u);
}

#[test]
#[should_fail]
fn test_compile_bad_pattern2() {
    drop(Pcre::compile("[").unwrap()); // Should be Err, will fail.
}

#[test]
fn test_compile_capture_count() {
    let re = Pcre::compile("(?:abc)(def)").unwrap();
    assert_eq!(re.capture_count(), 1u);
}

#[test]
fn test_exec_basic() {
    let mut re = Pcre::compile("^...$").unwrap();
    assert_eq!(re.capture_count(), 0u);
    let m = re.exec("abc").unwrap();
    assert_eq!(m.group(0), "abc");
}

#[test]
fn test_exec_no_match() {
    let mut re = Pcre::compile("abc").unwrap();
    assert!(re.exec("def").is_none());
}

#[test]
fn test_exec_nul_byte() {
    // Nul bytes *are* allowed in subject strings, however.
    let mut re = Pcre::compile("abc\\0def").unwrap();
    let m = re.exec("abc\0def").unwrap();
    assert_eq!(m.group(0), "abc\0def");
}

#[test]
fn test_exec_from_basic() {
    let mut re = Pcre::compile("abc").unwrap();
    let subject = "abcabc";
    let m1 = re.exec_from(subject, 1u).unwrap();
    assert_eq!(m1.group_start(0u), 3u);
    assert_eq!(m1.group_end(0u), 6u);
    assert_eq!(m1.group_len(0u), 3u);
    let m2 = re.exec(subject).unwrap();
    assert_eq!(m2.group_start(0u), 0u);
}

#[test]
fn test_study_basic() {
    let mut re = Pcre::compile("abc").unwrap();
    let mut study_res = re.study();
    assert!(study_res);
    // Re-study the pattern two more times (to check for leaks when the test program
    // is run through Valgrind).
    study_res = re.study();
    assert!(study_res);
    study_res = re.study();
    assert!(study_res);
}

#[test]
fn test_matches_basic() {
    let subject = "\0abc1111abcabc___ababc+a";
    let mut it = {
        let re = Pcre::compile("abc").unwrap();
        re.matches(subject)

        // The MatchIterator should retain a reference to the `pcre`.
    };

    let mut opt_m = it.next();
    assert!(opt_m.is_some());
    let mut m = opt_m.unwrap();
    assert_eq!(m.group_start(0u), 1u);
    assert_eq!(m.group_end(0u), 4u);

    let opt_m2 = it.next();
    assert!(opt_m2.is_some());
    let m2 = opt_m2.unwrap();
    assert_eq!(m2.group_start(0u), 8u);
    assert_eq!(m2.group_end(0u), 11u);
    // Verify that getting the next match has not changed the first match data.
    assert_eq!(m.group_start(0u), 1u);
    assert_eq!(m.group_end(0u), 4u);

    opt_m = it.next();
    assert!(opt_m.is_some());
    m = opt_m.unwrap();
    assert_eq!(m.group_start(0u), 11u);
    assert_eq!(m.group_end(0u), 14u);

    opt_m = it.next();
    assert!(opt_m.is_some());
    m = opt_m.unwrap();
    assert_eq!(m.group_start(0u), 19u);
    assert_eq!(m.group_end(0u), 22u);

    opt_m = it.next();
    assert!(opt_m.is_none());
}

#[test]
fn test_extra_mark() {
    let pattern = "X(*MARK:A)Y|X(*MARK:B)Z";
    let subject1 = "XY";
    let subject2 = "XZ";

    let mut compile_options: EnumSet<CompileOption> = EnumSet::empty();
    compile_options.add(pcre::Extra);

    let mut re = Pcre::compile_with_options(pattern, &compile_options).unwrap();

    // first try to get the mark from the compile to make sure it fails
    assert_eq!(re.get_mark(), None);

    let mut study_options: EnumSet<StudyOption> = EnumSet::empty();
    study_options.add(pcre::StudyJitCompile);
    let study = re.study_with_options(&study_options);
    // Double check to make sure the study worked
    assert!(study);

    // Now after studying, we still should not be able to get the mark (since we still need 
    // to set the option in the extra AND execute it)
    assert_eq!(re.get_mark(), None);

    // set that I am using the extra mark field
    let mut extra_options: EnumSet<ExtraOption> = EnumSet::empty();
    extra_options.add(pcre::ExtraMark);
    let extra = re.set_extra_options(&extra_options);
    // This will fail only if I didn't study first
    assert!(extra);

    // We still haven't run the pcre_exec yet so get mark should be None still
    assert_eq!(re.get_mark(), None);

    // Now execute and we should be able to get the mark
    let opt_m1 = re.exec(subject1);
    assert!(opt_m1.is_some());

    // It should match XY 
    let m1 = opt_m1.unwrap();
    assert_eq!(m1.group(0), "XY");

    // and the marked value should be A
    let mark1 = re.get_mark();
    assert!(mark1.is_some());
    assert_eq!(mark1.unwrap(), ~"A");

    let opt_m2 = re.exec(subject2);
    assert!(opt_m2.is_some());

    let m2 = opt_m2.unwrap();
    // It should match XZ
    assert_eq!(m2.group(0), "XZ");

    // and the marked value should be B
    assert_eq!(re.get_mark().unwrap(), ~"B");
}