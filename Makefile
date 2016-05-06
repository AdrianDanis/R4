#TODO: Refactor some of this to support options etc for other architectures
ARCH := x86_64
TARGET := $(ARCH)-unknown-r4-nofloat
LDFLAGS := -nostdlib -Wl,-n -Wl,--gc-sections -Wl,--build-id=none -Wl,-z,max-page-size=0x1000
ASFLAGS :=
CARGOFLAGS :=
RUSTFLAGS := -Z no-landing-pads -C no-stack-check
GCC := gcc
DEBUG ?= y
RUSTDOC=$(shell pwd)/rustdoc.sh
RUST_TARGET_PATH=$(shell pwd)
export RUST_TARGET_PATH

export RUSTDOC

ifeq ($(DEBUG), y)
	KERNEL_LIB := target/$(TARGET)/debug/libr4.a
	RUST_DEPS_DIR := target/$(TARGET)/debug/deps
else
	KERNEL_LIB := target/$(TARGET)/release/libr4.a
	RUST_DEPS_DIR := target/$(TARGET)/release/deps
	CARGOFLAGS += --release
endif

KERNEL := build/kernel-$(ARCH)
LINKER_SCRIPT := src/arch/$(ARCH)/linker.ld

# Grab any architecture assembly files
afiles := $(wildcard src/arch/$(ARCH)/*.S)
ofiles := $(patsubst src/arch/$(ARCH)/%.S, \
    build/arch/$(ARCH)/%.o, $(afiles))

.PHONY: lib all clean

all: $(KERNEL)

$(KERNEL): $(KERNEL)-elf64
	objcopy --strip-unneeded -O elf32-i386 $< $@

$(KERNEL)-elf64: $(ofiles) $(LINKER_SCRIPT) lib
	$(GCC) $(LDFLAGS) -T $(LINKER_SCRIPT) -o $@ $(ofiles) $(KERNEL_LIB) $(wildcard $(RUST_DEPS_DIR)/*)

lib:
	cargo rustc --target $(TARGET) $(CARGOFLAGS) --verbose -- $(RUSTFLAGS)

build/arch/$(ARCH)/%.o: src/arch/$(ARCH)/%.S
	mkdir -p $(shell dirname $@)
	$(GCC) -c $(ASFLAGS) $< -o $@

clean:
	cargo clean
	rm -rf build

doc: lib
	cargo doc
