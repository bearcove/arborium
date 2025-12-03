with Ada.Text_IO;               use Ada.Text_IO;
with Ada.Integer_Text_IO;       use Ada.Integer_Text_IO;
with Ada.Containers.Vectors;

procedure Inventory is
   type Item is record
      Name  : String (1 .. 24);
      Count : Integer := 0;
   end record;

   function To_Name (S : String) return String is
      Result : String (1 .. 24) := (others => ' ');
   begin
      Result (Result'First .. Result'First + S'Length - 1) := S;
      return Result;
   end To_Name;

   package Item_Vectors is new Ada.Containers.Vectors
     (Index_Type   => Positive,
      Element_Type => Item);
   use Item_Vectors;

   Inventory : Vector;

   procedure Add (Label : String; N : Integer) is
   begin
      for I in Inventory.First .. Inventory.Last loop
         if Inventory (I).Name = To_Name (Label) then
            Inventory (I).Count := Inventory (I).Count + N;
            return;
         end if;
      end loop;
      Inventory.Append (Item'(Name  => To_Name (Label),
                             Count => N));
   end Add;

   procedure Remove (Label : String; N : Integer) is
   begin
      for I in Inventory.First .. Inventory.Last loop
         if Inventory (I).Name = To_Name (Label) then
            Inventory (I).Count := Inventory (I).Count - N;
            exit;
         end if;
      end loop;
   end Remove;

   procedure Dump is
   begin
      Put_Line ("=== Inventory ===");
      for I in Inventory.First .. Inventory.Last loop
         Put ("* ");
         Put (Inventory (I).Name);
         Put (" => ");
         Put (Inventory (I).Count, Width => 0);
         New_Line;
      end loop;
      Put_Line ("==================");
   end Dump;

begin
   Add ("Hammers", 12);
   Add ("Nails",   500);
   Add ("Glue",    25);
   Remove ("Nails", 30);
   Add ("Hammers", 8);
   Dump;
end Inventory;
