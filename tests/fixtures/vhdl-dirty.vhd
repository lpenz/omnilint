library IEEE;
use IEEE.std_logic_1164.all;

entity demo is
    port (
        a : in  std_logic;
        b : out std_logic
    );
end entity;

architecture behaviour of demo is
begin
    a <= b;
end architecture;
