#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Cpu8080 Cpu8080;

typedef enum Message_Tag {
  Interrupt,
  Suspend,
  Restart,
  Shutdown,
} Message_Tag;

typedef struct Interrupt_Body {
  uint8_t irq_no;
  bool allow_nested_interrupt;
} Interrupt_Body;

typedef struct Message {
  Message_Tag tag;
  union {
    Interrupt_Body interrupt;
  };
} Message;

typedef struct CpuSender {
  struct Cpu8080 *cpu;
  void *sender;
} CpuSender;

typedef struct IoCallbacks {
  /**
   * IN port, pass port number back to app
   * set the calculated result back to reg_a
   */
  uint8_t (*input)(const void *io_object, uint8_t port);
  /**
   * OUT port value, pass port & value back to app
   */
  void (*output)(const void *io_object, uint8_t port, uint8_t value);
} IoCallbacks;

/**
 * # Safety
 * This function should be called with valid rom path
 * and the RAM will be allocated on the fly
 */
struct CpuSender new_cpu_instance(const char *rom_path,
                                  uintptr_t ram_size,
                                  struct IoCallbacks callbacks,
                                  const void *io_object);

/**
 * # Safety
 * This function should be safe to start a run loop.
 * Send a `Shutdown` message can break the loop, so
 * that the CPU and the Sender will be dropped, this is
 * the only way to release the resources to the system.
 */
void run(struct Cpu8080 *cpu, void *sender);

/**
 * # Safety
 * This function should be safe for accessing video ram.
 */
const uint8_t *get_ram(struct Cpu8080 *cpu);

/**
 * # Safety
 * Sender needs to be present(not dropped) for
 * sending the messages to the CPU instance.
 */
void send_message(void *sender, struct Message message);
