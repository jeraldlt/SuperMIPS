.data
	title: .asciiz "Bouncy Square"
	msg_1: .asciiz "The square has bounced "
	msg_2: .asciiz " times.\n"
.text
	li $t0, 64         # t0 contains square size
	li $t1, 0xFF0000FF # t1 contains square color
	li $t2, 0x0040     # t2 contains square x position (upper left corner)
	li $t3, 0x0080     # t3 contains square y position (upper left corner)
	li $t4, 0x0001     # t4 contains square x velocity
	li $t5, 0xFFFF     # t5 contains square y velocity
	li $t6, 0x777777FF # t6 contains background color
	
	li $t9, 0xFFFFFFFF # t9 contains -1
	
	move $s0, $zero    # s0 contains the number of bounces
	li $s1, 640        # s1 contains the image width
	li $s2, 480        # s2 contains the image height
	
	# Set the title
	li $v0, 0x05
	la $a0, title
	syscall
	
	# Seed RNG
	li $v0, 0x09
	syscall
loop:
	# Calculate lower-left corner
	addu $t7, $t2, $t0 # lower-left x position
	addu $t8, $t3, $t0 # lower-left y position
	
	# Determine if x velocity change is needed
	move $s7, $zero
	beq $t2, $zero, flip_x
	blt $t7, $s1, done_x
flip_x:
	# Multiply x velocity by 1
	xor $t4, $t4, $t9
	addi $t4, $t4, 1
	li $s7, 1
	
done_x:
	# Determine if y velocity change is needed
	beq $t3, $zero, flip_y
	blt $t8, $s2, done_y
flip_y:
	# Multiply y velocity by 1
	xor $t5, $t5, $t9
	addi $t5, $t5, 1
	li $s7, 1
done_y:
	beq $s7, $zero, done_bounce
	# Get random color for square
	li $v0, 0x0A
	syscall
	move $t1, $v0
	ori $t1, $t1, 0x00FF # Make sure alpha is always FF
	
	# Increase bounce count
	addi $s0, $s0, 1
	# Print message
	li $v0, 0x03
	la $a0, msg_1
	syscall
	
	li $v0, 0x04
	move $a0, $s0
	li $a1, 1
	syscall
	
	li $v0, 0x03
	la $a0, msg_2
	syscall
	
done_bounce:

	# Move square
	addu $t2, $t2, $t4
	andi $t2, $t2, 0xFFFF
	addu $t3, $t3, $t5
	andi $t3, $t3, 0xFFFF
	
	# Recalculate lower-left corner
	addu $t7, $t2, $t0 # lower-left x position
	addu $t8, $t3, $t0 # lower-left y position
	
	# Fill the screen with background color
	li $v0, 0x20
	move $a0, $t6
	syscall
	
	# Draw the square
	li $v0, 0x22
	move $a0, $t1     # Set the square color
	sll $a1, $t2, 16  # Top-left x position
	or $a1, $a1, $t3  # Top-left y position
	sll $a2, $t7, 16  # Bottom-right x position
	or $a2, $a2, $t8  # Bottom-right y position
	syscall
	
	# Update the frame
	li $v0, 0x01
	syscall
	
	# Sync the frame
	li $v0, 0x02
	syscall
	
	li $v0, 0x12
	syscall
	andi $v0, $v0, 1
	beq $v0, $zero, loop
	li $v0, 0
	syscall
	
	j loop

