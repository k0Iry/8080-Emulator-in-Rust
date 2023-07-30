#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Cpu8080 Cpu8080;

typedef struct IoCallbacks {
  /**
   * IN port, pass port number back to app
   * set the calculated result back to reg_a
   */
  uint8_t (*input)(uint8_t port);
  /**
   * OUT port value, pass port & value back to app
   */
  void (*output)(uint8_t port, uint8_t value);
} IoCallbacks;

typedef struct IrqMessage {
  uint8_t irq_no;
  bool allow_nested_interrupt;
} IrqMessage;

typedef enum Message_Tag {
  Interrupt,
  ExecutionControl,
} Message_Tag;

typedef struct Message {
  Message_Tag tag;
  union {
    struct {
      struct IrqMessage interrupt;
    };
  };
} Message;

/**
 * # Safety
 * This function should be called with valid rom path
 * and the RAM will be allocated on the fly
 */
struct Cpu8080 *new_cpu_instance(const char *rom_path,
                                 uintptr_t ram_size,
                                 struct IoCallbacks callbacks);

/**
 * # Safety
 * This function should be safe
 */
void run(struct Cpu8080 *cpu);

/**
 * Always called from a separated thread!
 * It is crucial that we don't borrow our CPU instance
 * since this function will be called from FFI thread.
 * (e.g. threads spawned by Swift language where we
 * cannot enforce any ownership mechanism)
 * Channel for the control of execution, we can either start
 * or pause the execution of instructions, again we
 * shall not borrow the CPU instance same as `send_interrupt`
 * since this function should always be called from a separated thread
 * # Safety
 * This function should be safe for accessing video ram
 */
const uint8_t *get_ram(struct Cpu8080 *cpu);

void send_message(struct Message message);
