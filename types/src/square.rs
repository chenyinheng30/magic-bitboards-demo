use super::*;

macro_rules! simple_enum {
    ($(
        pub enum $name:ident {
            $($variant:ident),*
        }
    )*) => {$(
        #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $name {
            $($variant),*
        }

        impl $name {
            pub const NUM: usize = [$(Self::$variant),*].len();
            pub const ALL: [Self; Self::NUM] = [$(Self::$variant),*];

            pub fn try_index(index: usize) -> Option<Self> {
                $(#[allow(non_upper_case_globals, unused)]
                const $variant: usize = $name::$variant as usize;)*
                #[allow(non_upper_case_globals)]
                match index {
                    $($variant => Option::Some(Self::$variant),)*
                    _ => Option::None
                }
            }

            pub fn index(index: usize) -> Self {
                Self::try_index(index).unwrap_or_else(|| panic!("Index {} is out of range.", index))
            }
        }
    )*};
}

simple_enum! {
    pub enum File {
        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I
    }

    pub enum Rank {
        First,
        Second,
        Third,
        Fourth,
        Fifth,
        Sixth,
        Seventh,
        Eighth,
        Nineth,
        Tenth
    }

    pub enum Square {
        A1, B1, C1, D1, E1, F1, G1, H1, I1,
        A2, B2, C2, D2, E2, F2, G2, H2, I2,
        A3, B3, C3, D3, E3, F3, G3, H3, I3,
        A4, B4, C4, D4, E4, F4, G4, H4, I4,
        A5, B5, C5, D5, E5, F5, G5, H5, I5,
        A6, B6, C6, D6, E6, F6, G6, H6, I6,
        A7, B7, C7, D7, E7, F7, G7, H7, I7,
        A8, B8, C8, D8, E8, F8, G8, H8, I8,
        A9, B9, C9, D9, E9, F9, G9, H9, I9,
        A0, B0, C0, D0, E0, F0, G0, H0, I0
    }
}

impl Square {
    pub fn new(file: File, rank: Rank) -> Self {
        Self::index(file as usize + rank as usize * 9)
    }

    pub fn file(self) -> File {
        File::index(self as usize % 9)
    }

    pub fn rank(self) -> Rank {
        Rank::index(self as usize / 9)
    }

    pub fn bitboard(self) -> BitBoard {
        BitBoard(1u128 << self as usize)
    }

    pub fn try_offset(self, file_offset: i8, rank_offset: i8) -> Option<Square> {
        Some(Square::new(
            File::try_index((self.file() as i8 + file_offset).try_into().ok()?)?,
            Rank::try_index((self.rank() as i8 + rank_offset).try_into().ok()?)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_file() {
        let s1 = Square::new(File::A, Rank::First);
        let s2 = Square::new(File::E, Rank::Fifth);
        let s3 = Square::new(File::I, Rank::Tenth);
        assert!(s1.file() == File::A, "fail with file = {:?}!", s1.file());
        assert!(s2.file() == File::E, "fail with file = {:?}!", s2.file());
        assert!(s3.file() == File::I, "fail with file = {:?}!", s3.file());
    }

    #[test]
    fn test_square_rank() {
        let s1 = Square::new(File::A, Rank::First);
        let s2 = Square::new(File::E, Rank::Fifth);
        let s3 = Square::new(File::I, Rank::Tenth);
        assert!(
            s1.rank() == Rank::First,
            "fail with rank = {:?}!",
            s1.rank()
        );
        assert!(
            s2.rank() == Rank::Fifth,
            "fail with rank = {:?}!",
            s2.rank()
        );
        assert!(s3.rank() == Rank::Tenth, "fail with rank = {:?}!", s3.rank());
    }

    #[test]
    fn test_square_bitboard() {
        let board = bitboard! {
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . X . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
        };
        let square = Square::new(File::D, Rank::Seventh);
        assert!(
            square.bitboard() == board,
            "fail with square = {:?}!",
            square
        );
    }

    #[test]
    fn test_square_try_offset0() {
        let square = Square::new(File::E, Rank::Third);
        let o1 = square.try_offset(-1, 0).unwrap();
        let o2 = square.try_offset(0, 1).unwrap();
        let o3 = square.try_offset(1, 2).unwrap();
        let o4 = square.try_offset(-2, 1).unwrap();
        assert!(o1 == Square::new(File::D, Rank::Third));
        assert!(o2 == Square::new(File::E, Rank::Fourth));
        assert!(o3 == Square::new(File::F, Rank::Fifth));
        assert!(o4 == Square::new(File::C, Rank::Fourth));
    }

    #[test]
    fn test_square_try_offset1() {
        let square = Square::new(File::H, Rank::Nineth);
        let o1 = square.try_offset(2, 0);
        let o2 = square.try_offset(5, 0);
        let o3 = square.try_offset(0, 2);
        assert!(o1 == None);
        assert!(o2 == None);
        assert!(o3 == None);
    }
}
