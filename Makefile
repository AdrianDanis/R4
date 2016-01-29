#TODO: Refactor some of this to support options etc for other architectures
ARCH := x86_64
TARGET := $(ARCH)-r4-none
LDFLAGS := -nostdlib -Wl,-n -Wl,--gc-sections -Wl,--build-id=none
ASFLAGS :=
CARGOFLAGS :=
RUSTFLAGS := -Z no-landing-pads
GCC := gcc
DEBUG := y

ifeq ($(DEBUG), y)
	KERNEL_LIB := target/$(TARGET)/debug/libr4.a
else
	KERNEL_LIB := target/$(TARGET)/release/libr4.a
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
	objcopy -O elf32-i386 $< $@

$(KERNEL)-elf64: $(ofiles) lib $(LINKER_SCRIPT) lib
	$(GCC) $(LDFLAGS) -T $(LINKER_SCRIPT) -o $@ $(ofiles) $(KERNEL_LIB)

lib:
	cargo rustc --target $(TARGET) --features disable_float $(CARGOFLAGS) --verbose -- $(RUSTFLAGS)

build/arch/$(ARCH)/%.o: src/arch/$(ARCH)/%.S
	mkdir -p $(shell dirname $@)
	$(GCC) -c $(ASFLAGS) $< -o $@

clean:
	cargo clean
	rm -rf build
