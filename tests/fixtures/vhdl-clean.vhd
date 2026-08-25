library IEEE;
use IEEE.std_logic_1164.all;

entity or_gate is
    port (
        a : in  std_logic;
        b : in  std_logic;
        y : out std_logic
    );
end entity;

architecture behaviour of or_gate is
begin
    y <= a or b;
end architecture;
