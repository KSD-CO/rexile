# ReXile vs Regex Benchmark Analysis

## Tính khách quan của Benchmarks

### Kết quả thực tế đã đo được:

#### 1. Literal Search (tìm chuỗi đơn giản "fox")
```
ReXile: 14.3-14.6 ns
regex:  11.8-12.3 ns
→ regex nhanh hơn ~20% (1.2x)
```

**Tại sao regex nhanh hơn?**
- regex crate có JIT compilation cho patterns đơn giản
- ReXile có overhead từ Pattern wrapper và enum dispatch
- Cho patterns cực kỳ đơn giản, overhead của abstraction lớn hơn lợi ích

#### 2. Multi-Pattern (2 patterns: "import|export")
```
ReXile: 15.8-16.2 ns
regex:  20.9-21.4 ns
→ ReXile nhanh hơn ~32% (1.3x)
```

**Tại sao ReXile nhanh hơn?**
- aho-corasick được tối ưu cho multi-pattern matching
- regex phải compile thành alternation trong NFA/DFA
- Lợi thế càng lớn khi số patterns tăng

#### 3. Multi-Pattern (4 patterns: "import|export|function|return")
```
ReXile: ~18-20 ns (ước tính từ trend)
regex:  ~25-30 ns (ước tính từ trend)
→ ReXile nhanh hơn ~35-40%
```

## Đánh giá tính khách quan

### ✅ Điểm mạnh của benchmark:

1. **Sử dụng Criterion** - công cụ benchmark chuẩn của Rust:
   - Tự động làm warm-up
   - Phát hiện outliers
   - Tính confidence interval
   - So sánh với baseline

2. **Cùng điều kiện**:
   - Cùng text input
   - Cùng pattern
   - Cùng optimization level (--release)
   - Pre-compile patterns (không tính compile time vào match time)

3. **Đo đúng thứ đúng**:
   - Đo `is_match()` vs `is_match()` (cùng operation)
   - Sử dụng `black_box()` để ngăn compiler optimization
   - Multiple samples (20-100 samples)

### ⚠️ Điểm cần lưu ý:

1. **Benchmark ngắn (nanoseconds)**:
   - Sai số đo lường cao hơn
   - Noise từ system có thể ảnh hưởng
   - Cần nhiều iterations để có kết quả ổn định

2. **Test cases đơn giản**:
   - Chỉ test với text ngắn (~80 chars)
   - Patterns đơn giản (literals, alternations)
   - Chưa test với text lớn (KB-MB)

3. **Không test edge cases**:
   - Empty patterns
   - Very long patterns
   - Unicode text
   - Pathological cases

## Kết luận về tính khách quan

### Benchmark KHÁCH QUAN với những giới hạn:

**✅ Khách quan:**
- Sử dụng công cụ chuẩn (Criterion)
- Điều kiện đo lường công bằng
- Kết quả có thể reproduce
- Không cherry-pick results

**⚠️ Giới hạn:**
- **ReXile KHÔNG phải lúc nào cũng nhanh hơn regex**
- Cho literal đơn giản, regex JIT nhanh hơn
- ReXile có lợi thế với multi-pattern (2+ patterns)
- Trade-off: ReXile đơn giản hơn nhưng ít tính năng hơn

## Recommendations

### Khi nào dùng ReXile?
- ✅ Multi-pattern matching (3+ patterns)
- ✅ Cần compilation nhanh
- ✅ Muốn dependencies nhỏ gọn
- ✅ Patterns đơn giản (literals, anchors, alternation)

### Khi nào dùng regex crate?
- ✅ Cần full regex features (char classes, quantifiers, lookahead)
- ✅ Single literal patterns (có JIT optimization)
- ✅ Capture groups
- ✅ Complex patterns

## Cách cải thiện ReXile

Để cạnh tranh với regex cho literal search:

1. **Inline fast path** cho single literal:
   ```rust
   // Thay vì enum dispatch, inline memchr::memmem trực tiếp
   if pattern.is_literal() {
       memchr::memmem::find(...)  // Inline, không qua enum
   }
   ```

2. **SIMD optimization** cho patterns ngắn:
   ```rust
   // Dùng SIMD cho patterns 2-16 bytes
   if pattern.len() <= 16 {
       simd_search(...)
   }
   ```

3. **JIT compilation** cho hot patterns (advanced):
   - Cache machine code cho patterns thường dùng
   - Trade-off: tăng complexity

## Verdict

**Benchmarks có khách quan** nhưng kết quả cho thấy:
- regex crate vẫn tối ưu hơn cho simple cases
- ReXile có niche riêng: multi-pattern matching
- Không nên claim "ReXile luôn nhanh hơn regex"
- Nên claim "ReXile tối ưu cho multi-pattern và có API đơn giản hơn"

---

**Honest marketing:**
> "ReXile: A minimal regex-lite engine optimized for multi-pattern matching. 
> For simple literal searches, regex's JIT is faster. 
> For 3+ patterns, ReXile's aho-corasick backend shines."
