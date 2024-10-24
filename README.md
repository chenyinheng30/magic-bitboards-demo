# Magic Bitboards demo

用于中国象棋 magic 数字生成，用于象棋中车、炮、马、象（相）、士（仕）、将（帅）以及红方兵和黑方卒的走法生成。基于 magic bitboard 和完美哈希，受国际象棋启发，参考[国际象棋类似的程序](https://analog-hors.github.io/writing/magic-bitboards/)。

## 如何使用

单个棋盘使用128位无符号整数表示，只使用其中的低90位，其余高位填充为0。表示局面的结构体至少使用使用9个棋盘分别表示不同的颜色和不同的兵种。

1. 表示红色棋子位置和黑色棋子位置ge一个
2. 表示将（帅）、车、马、炮、象（相）、士、卒（兵）不同兵种棋盘各一个不区分颜色。

低90位中红方位于低45位，黑方位于高45位。可以添加附加信息如未吃子行棋数等辅助生成正确走法。

使用Magic Bitboards只能用于生成依赖当前局面的走法，需要配合辅助信息保证行棋符合规定。简单介绍Magic Bitboards，这是使用与棋子位置、当前局面以及一个Magic数字相关的函数计算走法在哈希表中索引

```rust
pub fn magic_index(entry: &MagicEntry, blockers: BitBoard) -> usize {
    let blockers = blockers & entry.mask;
    let hash = blockers.0.wrapping_mul(entry.magic as u128);
    let index = (hash.wrapping_shr(entry.shift.into())) as usize;
    index
}
```

`MagicEntry`结构体包含掩模、Magic数字、移位数三个变量，得到索引还需要加上偏移量。

```rust
pub struct MagicEntry {
    pub mask: BitBoard,
    pub magic: u128,
    pub shift: u8,
}
```

1. 掩模：提取走法生成中关心的位置。
2. Magic数字：将局面映射到索引。
3. 移位数：减少表大小。

具体介绍生成Magic数字时不同兵种的走法算法：

车、炮、马、象（相）：不区分颜色，直接按象棋行棋规则生成走法。

卒（兵）、士：不使用Magic数字。

将（帅）：颜色按位置自然区分，结果包括走法和将、帅不准对面的禁点。

实际生成走法时使用的算法：

车、炮、马、象（相）：1、根据位置获取 `MagicEntry`结构体。2、计算走法的哈希索引。3、获取哈希表中走法集合。

卒（兵）、士：直接以位置作为哈希，获取哈希表中走法走法集合。

将（帅）：1、生成将的可能的走法。2、生成对手所有棋子走法和对手帅（将）的禁点，统一按位于。3、两者走法掩模得到禁点，使得将、帅不对面，将（帅）不送将。4、异或去掉走法中的禁点，得到走法集合。

因为障碍没考虑颜色，所以最后还需要去除和己方棋子位置掩模再异或去除走到己方已占据位置的点，最后遍历比特获取走法，更新局面。
