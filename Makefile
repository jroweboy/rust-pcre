PCRE_LIBDIR ?= $(shell pcre-config --prefix)/lib

PCRE_LIBVERSION_REQUIRED ?= 830
# see stackoverflow http://stackoverflow.com/q/5188267/745719
PCRE_LIBVERSION_GTE ?= $(shell expr `pcre-config --version | sed -e 's/\.\([0-9][0-9]\)/\1/g'` \>= $(PCRE_LIBVERSION_REQUIRED))

LINKFLAGS ?= -L lib -L "$(PCRE_LIBDIR)"
RUSTFLAGS ?= -O
CRATETYPE ?=  --crate-type=dylib,rlib 

.PHONY: all demo clean test doc

VERSION ?= 0.1
LIB_OUTNAME ?= lib/libpcre-ab318eaa-0.1.so
TEST_OUTNAME ?= build/libtest~

src_files=\
			src/pcre/mod.rs\
			src/pcre/detail/mod.rs\
			src/pcre/detail/native.rs

demo_files=\
			src/pcredemo/main.rs

all: $(LIB_OUTNAME) test

$(LIB_OUTNAME): $(src_files)
# if they don't have PCRE >= 8.30
ifneq ("$(PCRE_LIBVERSION_GTE)", "1")
	# $(shell echo "test $(PCRE_LIBVERSION_GTE)")
	$(error The installed pcre version $(shell echo `pcre-config --version`) is too low. Version >= 8.30 is required)
endif
	mkdir -p lib/
	rustc $(RUSTFLAGS) $(CRATETYPE) $(LINKFLAGS) src/pcre/mod.rs --out-dir=lib

demo:
	mkdir -p build/
	rustc $(RUSTFLAGS) $(LINKFLAGS) src/pcredemo/main.rs --out-dir build/

test: $(LIB_OUTNAME) $(TEST_OUTNAME)

$(TEST_OUTNAME): src/pcre/test.rs
	mkdir -p build/
	rustc $(RUSTFLAGS) $(LINKFLAGS) --test src/pcre/test.rs -o build/libtest~

doc:
	rustdoc --output doc -w html src/pcre/mod.rs

clean:
	$(RM) -r .rust bin build lib libtest~ libtest~.dSYM

# keeping this around for no apparent reason
install: old_install


old_install:
	test -d build || mkdir build
	rustc --out-dir build pkg.rs && ./build/pkg

old_test:
	test -d build || mkdir build
	rustc --test src/pcre/test.rs -o build/libtest~ -L lib -L "$(PCRE_LIBDIR)" && ./build/libtest~
