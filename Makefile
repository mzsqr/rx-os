

fs.img: rx-user/*
	cd rx-user && make fs.img
# xv6-mkfs/mkfs fs.img README.md $(UEXTRA) $(UPROGS) hello

clean:
	rm -rf *.tex *.dvi *.idx *.aux *.log *.ind *.ilg *.dSYM *.zip *.pcap \
	*/*.o */*.d */*.asm */*.sym target \
	$U/initcode $U/initcode.out $U/usys.S $U/_* \
	mkfs/mkfs fs.img fs.img.bk .gdbinit __pycache__ xv6.out* \
	ph barrier

qemu: fs.img
	cd rx-kernel && cargo run --bin kernel
