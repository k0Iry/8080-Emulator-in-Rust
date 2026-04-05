#ifndef I8080EMULATOR_FFI_H
#define I8080EMULATOR_FFI_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct I8080Cpu I8080Cpu;

typedef enum I8080Status {
  I8080Status_Ok = 0,
  I8080Status_NullPointer = 1,
  I8080Status_NoPendingInput = 2,
  I8080Status_MemoryOutOfBounds = 3,
  I8080Status_InvalidInterrupt = 4,
  I8080Status_InvalidArguments = 5,
  I8080Status_InvalidAlignment = 6,
} I8080Status;

typedef enum I8080ExecutionState {
  I8080ExecutionState_Continue = 0,
  I8080ExecutionState_Input = 1,
  I8080ExecutionState_Output = 2,
  I8080ExecutionState_Halted = 3,
} I8080ExecutionState;

typedef struct I8080StepResult {
  uint64_t cycles;
  enum I8080ExecutionState state;
  uint8_t port;
  uint8_t value;
} I8080StepResult;

size_t i8080_cpu_size(void);
size_t i8080_cpu_align(void);

I8080Status i8080_init_cpu(void *storage,
                           const uint8_t *rom_ptr,
                           size_t rom_len,
                           uint8_t *ram_ptr,
                           size_t ram_len,
                           I8080Cpu **out_cpu);
I8080Status i8080_deinit_cpu(I8080Cpu *cpu);

I8080Status i8080_run(I8080Cpu *cpu, I8080StepResult *result);
I8080Status i8080_step(I8080Cpu *cpu, I8080StepResult *result);
I8080Status i8080_provide_input(I8080Cpu *cpu, uint8_t value);
I8080Status i8080_interrupt(I8080Cpu *cpu, uint8_t irq_no);
I8080Status i8080_restart(I8080Cpu *cpu);

bool i8080_is_halted(const I8080Cpu *cpu);
const uint8_t *i8080_ram_ptr(const I8080Cpu *cpu);
uint8_t *i8080_ram_mut_ptr(I8080Cpu *cpu);
size_t i8080_ram_len(const I8080Cpu *cpu);

#endif
