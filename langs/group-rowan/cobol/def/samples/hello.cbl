      *> COBOL Sample Program - Hello World with Basic Operations
       IDENTIFICATION DIVISION.
       PROGRAM-ID. HELLO-WORLD.
       AUTHOR. ARBORIUM.
       DATE-WRITTEN. 2024-01-01.

       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. IBM-PC.
       OBJECT-COMPUTER. IBM-PC.

       DATA DIVISION.
       WORKING-STORAGE SECTION.

      *> Numeric variables
       01 WS-COUNTER          PIC 9(3)    VALUE 0.
       01 WS-TOTAL            PIC 9(5)V99 VALUE 0.
       01 WS-RESULT           PIC Z,ZZ9.99.

      *> String variables
       01 WS-NAME             PIC X(30)   VALUE SPACES.
       01 WS-MESSAGE          PIC X(50).
       01 WS-GREETING         PIC X(20)   VALUE "Hello, ".

      *> Table (array) definition
       01 WS-TABLE.
          05 WS-ITEM          PIC X(10) OCCURS 5 TIMES
                              INDEXED BY WS-IDX.

      *> Record structure
       01 WS-EMPLOYEE.
          05 EMP-ID           PIC 9(6).
          05 EMP-NAME         PIC X(25).
          05 EMP-SALARY       PIC 9(7)V99.
          05 EMP-DEPT         PIC X(10).

       PROCEDURE DIVISION.
       MAIN-PROCEDURE.
           PERFORM INITIALIZE-DATA
           PERFORM DISPLAY-GREETING
           PERFORM CALCULATE-TOTALS
           PERFORM PROCESS-TABLE
           STOP RUN.

       INITIALIZE-DATA.
           MOVE "World" TO WS-NAME
           MOVE 100.50 TO WS-TOTAL
           MOVE 12345 TO EMP-ID
           MOVE "John Smith" TO EMP-NAME
           MOVE 75000.00 TO EMP-SALARY
           MOVE "IT" TO EMP-DEPT.

       DISPLAY-GREETING.
           STRING WS-GREETING DELIMITED BY SPACES
                  WS-NAME DELIMITED BY SPACES
                  "!" DELIMITED BY SIZE
                  INTO WS-MESSAGE
           END-STRING
           DISPLAY WS-MESSAGE
           DISPLAY "Employee: " EMP-NAME " ID: " EMP-ID.

       CALCULATE-TOTALS.
           PERFORM VARYING WS-COUNTER FROM 1 BY 1
                   UNTIL WS-COUNTER > 10
               ADD WS-COUNTER TO WS-TOTAL
           END-PERFORM
           MOVE WS-TOTAL TO WS-RESULT
           DISPLAY "Total: " WS-RESULT.

       PROCESS-TABLE.
           MOVE "First" TO WS-ITEM(1)
           MOVE "Second" TO WS-ITEM(2)
           MOVE "Third" TO WS-ITEM(3)

           PERFORM VARYING WS-IDX FROM 1 BY 1
                   UNTIL WS-IDX > 3
               DISPLAY "Item " WS-IDX ": " WS-ITEM(WS-IDX)
           END-PERFORM.

       EVALUATE-EXAMPLE.
           EVALUATE TRUE
               WHEN WS-COUNTER = 0
                   DISPLAY "Counter is zero"
               WHEN WS-COUNTER < 5
                   DISPLAY "Counter is less than 5"
               WHEN WS-COUNTER >= 5 AND WS-COUNTER <= 10
                   DISPLAY "Counter is between 5 and 10"
               WHEN OTHER
                   DISPLAY "Counter is greater than 10"
           END-EVALUATE.
