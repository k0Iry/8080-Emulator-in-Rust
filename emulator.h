#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Vec_u8 Vec_u8;

typedef uint8_t ConditionCodes;

typedef struct Cpu8080 {
  struct Vec_u8 *ram;
  const struct Vec_u8 *rom;
  uint16_t sp;
  uint16_t pc;
  uint8_t reg_a;
  uint8_t reg_b;
  uint8_t reg_c;
  uint8_t reg_d;
  uint8_t reg_e;
  uint8_t reg_h;
  uint8_t reg_l;
  ConditionCodes conditon_codes;
  bool interrupt_enabled;
} Cpu8080;

/**
 * # Safety
 * This function should be called with valid rom path
 * and the RAM will be allocated on the fly
 */
struct Cpu8080 *new_cpu_instance(const char *rom_path, uintptr_t ram_len);

/**
 * # Safety
 * This function should be safe
 */
void run(struct Cpu8080 *cpu);
