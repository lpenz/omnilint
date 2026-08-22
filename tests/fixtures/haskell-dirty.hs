module HaskellDirty where

f :: Maybe Int -> Int
f (Just x) = x
f Nothing = 0

g :: Maybe Int -> Int
g x = case x of
  Just y -> y
  Nothing -> 0

h xs = foldr (++) [] xs
