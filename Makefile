# SPDX-License-Identifier: GPL-2.0

KDIR ?= /lib/modules/`uname -r`/build

default:
	$(MAKE) -C $(KDIR) M=$$PWD

clean:
	rm -f *.ko *.mod *.mod.c *.o *.rmeta Module.symvers modules.order
