{-# LANGUAGE DeriveDataTypeable #-}
{-# LANGUAGE DeriveGeneric #-}

module Distribution.Types.Version
  ( -- * Package versions
    Version
  , mkVersion
  , mkVersion'
  , versionNumbers
  , nullVersion
  , alterVersion
  , version0

    -- * Internal
  , validVersion
  , versionDigitParser
  ) where

import Data.Bits (shiftL, shiftR, (.&.), (.|.))
import Distribution.Compat.Prelude
import Prelude ()

import Distribution.Parsec
import Distribution.Pretty

import qualified Data.Version as Base
import qualified Distribution.Compat.CharParsing as P
import qualified Text.PrettyPrint as Disp
import qualified Text.Read as Read

-- | A 'Version' represents the version of a software entity.
--
-- Instances of 'Eq' and 'Ord' are provided, which gives exact
-- equality and lexicographic ordering of the version number
-- components (i.e. 2.1 > 2.0, 1.2.3 > 1.2.2, etc.).
--
-- This type is opaque and distinct from the 'Base.Version' type in
-- "Data.Version" since @Cabal-2.0@. The difference extends to the
-- 'Binary' instance using a different (and more compact) encoding.
--
-- @since 2.0.0.2
data Version
  = PV0 {-# UNPACK #-} !Word64
  | PV1 !Int [Int]
  -- NOTE: If a version fits into the packed Word64
  -- representation (i.e. at most four version components
  -- which all fall into the [0..0xfffe] range), then PV0
  -- MUST be used. This is essential for the 'Eq' instance
  -- to work.
  deriving (Data, Eq, Generic)

instance Ord Version where
  compare (PV0 x) (PV0 y) = compare x y
  compare (PV1 x xs) (PV1 y ys) = case compare x y of
    EQ -> compare xs ys
    c -> c
  compare (PV0 w) (PV1 y ys) = case compare x y of
    EQ -> compare [x2, x3, x4] ys
    c -> c
    where
      x = fromIntegral ((w `shiftR` 48) .&. 0xffff) - 1
      x2 = fromIntegral ((w `shiftR` 32) .&. 0xffff) - 1
      x3 = fromIntegral ((w `shiftR` 16) .&. 0xffff) - 1
      x4 = fromIntegral (w .&. 0xffff) - 1
  compare (PV1 x xs) (PV0 w) = case compare x y of
    EQ -> compare xs [y2, y3, y4]
    c -> c
    where
      y = fromIntegral ((w `shiftR` 48) .&. 0xffff) - 1
      y2 = fromIntegral ((w `shiftR` 32) .&. 0xffff) - 1
      y3 = fromIntegral ((w `shiftR` 16) .&. 0xffff) - 1
      y4 = fromIntegral (w .&. 0xffff) - 1

instance Show Version where
  showsPrec d v =
    showParen (d > 10) $
      showString "mkVersion "
        . showsPrec 11 (versionNumbers v)

instance Read Version where
  readPrec = Read.parens $ do
    Read.Ident "mkVersion" <- Read.lexP
    v <- Read.step Read.readPrec
    return (mkVersion v)

instance Binary Version
instance Structured Version

instance NFData Version where
  rnf (PV0 _) = ()
  rnf (PV1 _ ns) = rnf ns

instance Pretty Version where
  pretty ver =
    Disp.hcat
      ( Disp.punctuate
          (Disp.char '.')
          (map Disp.int $ versionNumbers ver)
      )

instance Parsec Version where
  parsec = mkVersion <$> toList <$> P.sepByNonEmpty versionDigitParser (P.char '.') <* tags
    where
      tags = do
        ts <- many $ P.char '-' *> some (P.satisfy isAlphaNum)
        case ts of
          [] -> pure ()
          (_ : _) -> parsecWarning PWTVersionTag "version with tags"

-- | An integral without leading zeroes.
--
-- @since 3.0
versionDigitParser :: CabalParsing m => m Int
versionDigitParser = (some d >>= toNumber) P.<?> "version digit (integral without leading zeroes)"
  where
    toNumber :: CabalParsing m => [Int] -> m Int
    toNumber [0] = return 0
    toNumber (0 : _) = P.unexpected "Version digit with leading zero"
    toNumber xs
      -- 10^9 = 1000000000
