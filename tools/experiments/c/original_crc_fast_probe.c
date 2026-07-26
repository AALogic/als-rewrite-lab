#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
  const char *name;
  int width;
  uint32_t poly;
  uint32_t init;
  int refin;
  int refout;
  uint32_t xorout;
} crc_spec;

typedef struct {
  const char *name;
  const unsigned char *data;
  size_t len;
} context;

static uint32_t reflect_bits(uint32_t value, int bits) {
  uint32_t out = 0;
  for (int i = 0; i < bits; i++) {
    out = (out << 1) | (value & 1U);
    value >>= 1;
  }
  return out;
}

static uint32_t crc_compute(const crc_spec *spec, const unsigned char *data, size_t len) {
  uint32_t mask = spec->width == 32 ? 0xffffffffU : ((1U << spec->width) - 1U);
  uint32_t topbit = 1U << (spec->width - 1);
  uint32_t table[256];

  if (spec->refin) {
    uint32_t poly = reflect_bits(spec->poly, spec->width);
    for (int i = 0; i < 256; i++) {
      uint32_t crc = (uint32_t)i;
      for (int bit = 0; bit < 8; bit++) {
        crc = (crc & 1U) ? ((crc >> 1) ^ poly) : (crc >> 1);
      }
      table[i] = crc & mask;
    }
  } else {
    int shift = spec->width - 8;
    for (int i = 0; i < 256; i++) {
      uint32_t crc = (uint32_t)i << shift;
      for (int bit = 0; bit < 8; bit++) {
        crc = (crc & topbit) ? ((crc << 1) ^ spec->poly) : (crc << 1);
      }
      table[i] = crc & mask;
    }
  }

  uint32_t crc = spec->init & mask;
  if (spec->refin) {
    for (size_t i = 0; i < len; i++) {
      crc = ((crc >> 8) ^ table[(crc ^ data[i]) & 0xffU]) & mask;
    }
  } else {
    int shift = spec->width - 8;
    for (size_t i = 0; i < len; i++) {
      crc = ((crc << 8) ^ table[((crc >> shift) ^ data[i]) & 0xffU]) & mask;
    }
  }

  if (spec->refout != spec->refin) {
    crc = reflect_bits(crc, spec->width);
  }
  return (crc ^ spec->xorout) & mask;
}

static uint16_t bswap16(uint16_t value) {
  return (uint16_t)(((value & 0xffU) << 8) | ((value >> 8) & 0xffU));
}

static uint16_t sum_bytes16(const unsigned char *data, size_t len) {
  uint32_t sum = 0;
  for (size_t i = 0; i < len; i++) {
    sum += data[i];
  }
  return (uint16_t)(sum & 0xffffU);
}

static uint16_t sum_words16(const unsigned char *data, size_t len, int little) {
  uint32_t sum = 0;
  for (size_t i = 0; i < len; i += 2) {
    uint16_t word = 0;
    if (little) {
      word = data[i];
      if (i + 1 < len) word |= (uint16_t)data[i + 1] << 8;
    } else {
      word = (uint16_t)data[i] << 8;
      if (i + 1 < len) word |= data[i + 1];
    }
    sum += word;
  }
  return (uint16_t)(sum & 0xffffU);
}

static uint16_t fletcher16(const unsigned char *data, size_t len) {
  uint32_t sum1 = 0;
  uint32_t sum2 = 0;
  for (size_t i = 0; i < len; i++) {
    sum1 = (sum1 + data[i]) % 255U;
    sum2 = (sum2 + sum1) % 255U;
  }
  return (uint16_t)(((sum2 << 8) | sum1) & 0xffffU);
}

static uint32_t read_be32(const unsigned char *p) {
  return ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) | ((uint32_t)p[2] << 8) | p[3];
}

static uint32_t read_le32(const unsigned char *p) {
  return ((uint32_t)p[3] << 24) | ((uint32_t)p[2] << 16) | ((uint32_t)p[1] << 8) | p[0];
}

static int wav_payload(const unsigned char *data, size_t len, const unsigned char **out, size_t *out_len) {
  if (len < 12 || memcmp(data, "RIFF", 4) != 0 || memcmp(data + 8, "WAVE", 4) != 0) return 0;
  size_t pos = 12;
  while (pos + 8 <= len) {
    uint32_t size = read_le32(data + pos + 4);
    size_t start = pos + 8;
    size_t end = start + size;
    if (end > len) end = len;
    if (memcmp(data + pos, "data", 4) == 0) {
      *out = data + start;
      *out_len = end - start;
      return 1;
    }
    pos = end + (size % 2);
  }
  return 0;
}

static int aiff_payload(const unsigned char *data, size_t len, const unsigned char **out, size_t *out_len) {
  if (len < 12 || memcmp(data, "FORM", 4) != 0 ||
      (memcmp(data + 8, "AIFF", 4) != 0 && memcmp(data + 8, "AIFC", 4) != 0)) return 0;
  size_t pos = 12;
  while (pos + 8 <= len) {
    uint32_t size = read_be32(data + pos + 4);
    size_t start = pos + 8;
    size_t end = start + size;
    if (end > len) end = len;
    if (memcmp(data + pos, "SSND", 4) == 0 && end >= start + 8) {
      uint32_t offset = read_be32(data + start);
      size_t payload_start = start + 8 + offset;
      if (payload_start > end) payload_start = end;
      *out = data + payload_start;
      *out_len = end - payload_start;
      return 1;
    }
    pos = end + (size % 2);
  }
  return 0;
}

static unsigned char *read_file(const char *path, size_t *len) {
  FILE *file = fopen(path, "rb");
  if (!file) return NULL;
  if (fseek(file, 0, SEEK_END) != 0) { fclose(file); return NULL; }
  long size = ftell(file);
  if (size < 0) { fclose(file); return NULL; }
  rewind(file);
  unsigned char *data = (unsigned char *)malloc((size_t)size);
  if (!data) { fclose(file); return NULL; }
  if (fread(data, 1, (size_t)size, file) != (size_t)size) {
    free(data);
    fclose(file);
    return NULL;
  }
  fclose(file);
  *len = (size_t)size;
  return data;
}

int main(int argc, char **argv) {
  if (argc != 3) {
    fprintf(stderr, "usage: %s EXPECTED_CRC FILE\n", argv[0]);
    return 2;
  }

  uint16_t expected = (uint16_t)strtoul(argv[1], NULL, 10);
  size_t len = 0;
  unsigned char *data = read_file(argv[2], &len);
  if (!data) {
    fprintf(stderr, "failed to read %s\n", argv[2]);
    return 1;
  }

  const unsigned char *payload = NULL;
  size_t payload_len = 0;
  context contexts[2];
  int context_count = 1;
  contexts[0] = (context){"full_file", data, len};
  if (wav_payload(data, len, &payload, &payload_len)) {
    contexts[context_count++] = (context){"audio_payload_wav_data", payload, payload_len};
  } else if (aiff_payload(data, len, &payload, &payload_len)) {
    contexts[context_count++] = (context){"audio_payload_aiff_ssnd", payload, payload_len};
  }

  crc_spec specs[] = {
      {"crc16_arc", 16, 0x8005, 0x0000, 1, 1, 0x0000},
      {"crc16_modbus", 16, 0x8005, 0xffff, 1, 1, 0x0000},
      {"crc16_usb", 16, 0x8005, 0xffff, 1, 1, 0xffff},
      {"crc16_maxim", 16, 0x8005, 0x0000, 1, 1, 0xffff},
      {"crc16_ccitt_false", 16, 0x1021, 0xffff, 0, 0, 0x0000},
      {"crc16_xmodem", 16, 0x1021, 0x0000, 0, 0, 0x0000},
      {"crc16_kermit", 16, 0x1021, 0x0000, 1, 1, 0x0000},
      {"crc16_x25", 16, 0x1021, 0xffff, 1, 1, 0xffff},
      {"crc16_dnp", 16, 0x3d65, 0x0000, 1, 1, 0xffff},
      {"crc16_t10_dif", 16, 0x8bb7, 0x0000, 0, 0, 0x0000},
      {"crc16_dect_r", 16, 0x0589, 0x0000, 0, 0, 0x0001},
      {"crc16_cdma2000", 16, 0xc867, 0xffff, 0, 0, 0x0000},
      {"crc32_mpeg2", 32, 0x04c11db7, 0xffffffff, 0, 0, 0x00000000},
      {"crc32_bzip2", 32, 0x04c11db7, 0xffffffff, 0, 0, 0xffffffff},
      {"crc32c_castagnoli", 32, 0x1edc6f41, 0xffffffff, 1, 1, 0xffffffff},
      {"crc32_jamcrc", 32, 0x04c11db7, 0xffffffff, 1, 1, 0x00000000},
  };

  int matches = 0;
  for (int c = 0; c < context_count; c++) {
    context ctx = contexts[c];
    for (size_t i = 0; i < sizeof(specs) / sizeof(specs[0]); i++) {
      uint32_t value = crc_compute(&specs[i], ctx.data, ctx.len);
      uint16_t candidates[] = {
          (uint16_t)(value & 0xffffU),
          (uint16_t)((value >> 16) & 0xffffU),
          bswap16((uint16_t)(value & 0xffffU)),
          bswap16((uint16_t)((value >> 16) & 0xffffU)),
      };
      const char *suffixes[] = {"low16", "high16", "low16_byteswap", "high16_byteswap"};
      int suffix_count = specs[i].width == 16 ? 2 : 4;
      for (int s = 0; s < suffix_count; s++) {
        if (candidates[s] == expected) {
          printf("%s\t%s\t%s\t%u\t%s\n", argv[2], ctx.name, specs[i].name, candidates[s], suffixes[s]);
          matches++;
        }
      }
    }
    uint16_t sums[] = {
        sum_bytes16(ctx.data, ctx.len),
        sum_words16(ctx.data, ctx.len, 1),
        sum_words16(ctx.data, ctx.len, 0),
        fletcher16(ctx.data, ctx.len),
    };
    const char *sum_names[] = {"sum_bytes16", "sum_words16_little", "sum_words16_big", "fletcher16"};
    for (int i = 0; i < 4; i++) {
      if (sums[i] == expected) {
        printf("%s\t%s\t%s\t%u\tvalue\n", argv[2], ctx.name, sum_names[i], sums[i]);
        matches++;
      }
    }
  }

  free(data);
  return matches ? 0 : 3;
}
