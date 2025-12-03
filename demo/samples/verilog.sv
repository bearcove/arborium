// Copyright 2018 ETH Zurich and University of Bologna.
// Copyright and related rights are licensed under the Solderpad Hardware
// License, Version 0.51 (the "License"); you may not use this file except in
// compliance with the License.  You may obtain a copy of the License at
// http://solderpad.org/licenses/SHL-0.51. Unless required by applicable law
// or agreed to in writing, software, hardware and materials distributed under
// this License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.
//
// Author: Matthias Baer <baermatt@student.ethz.ch>
// Author: Igor Loi <igor.loi@unibo.it>
// Author: Andreas Traber <atraber@student.ethz.ch>
// Author: Lukas Mueller <lukasmue@student.ethz.ch>
// Author: Florian Zaruba <zaruabf@iis.ee.ethz.ch>
//
// Date: 19.03.2017
// Description: Ariane ALU based on RI5CY's ALU


module alu
  import ariane_pkg::*;
#(
    parameter config_pkg::cva6_cfg_t CVA6Cfg = config_pkg::cva6_cfg_empty,
    parameter bit HasBranch = 1'b1,
    parameter type fu_data_t = logic
) (
    // Subsystem Clock - SUBSYSTEM
    input logic clk_i,
    // Asynchronous reset active low - SUBSYSTEM
    input logic rst_ni,
    // FU data needed to execute instruction - ISSUE_STAGE
    input fu_data_t fu_data_i,
    // FU data needed to execute CPOP/CPOPW - ISSUE_STAGE
    input fu_data_t fu_data_cpop_i,
    // ALU result - ISSUE_STAGE
    output logic [CVA6Cfg.XLEN-1:0] result_o,
    // ALU branch compare result - branch_unit
    output logic alu_branch_res_o
);

  logic [CVA6Cfg.XLEN-1:0] operand_a_rev;
  logic [            31:0] operand_a_rev32;
  logic [  CVA6Cfg.XLEN:0] operand_b_neg;
  logic [CVA6Cfg.XLEN+1:0] adder_result_ext_o;
  logic                    less;  // handles both signed and unsigned forms
  logic [            31:0] rolw;  // Rotate Left Word
  logic [            31:0] rorw;  // Rotate Right Word
  logic [31:0] orcbw, rev8w;
  logic [  $clog2(CVA6Cfg.XLEN) : 0] cpop;  // Count Population
  logic [$clog2(CVA6Cfg.XLEN)-1 : 0] lz_tz_count;  // Count Leading Zeros
  logic [                       4:0] lz_tz_wcount;  // Count Leading Zeros Word
  logic lz_tz_empty, lz_tz_wempty;
  logic [CVA6Cfg.XLEN-1:0] orcbw_result, rev8w_result;

  logic [CVA6Cfg.XLEN-1:0] brev8_reversed;
  logic [            31:0] unzip_gen;
  logic [            31:0] zip_gen;
  logic [CVA6Cfg.XLEN-1:0] xperm8_result;
  logic [CVA6Cfg.XLEN-1:0] xperm4_result;

  // bit reverse operand_a for left shifts and bit counting
  generate
    genvar k;
    for (k = 0; k < CVA6Cfg.XLEN; k++)
      assign operand_a_rev[k] = fu_data_i.operand_a[CVA6Cfg.XLEN-1-k];

    for (k = 0; k < 32; k++) assign operand_a_rev32[k] = fu_data_i.operand_a[31-k];
  endgenerate

  // ------
  // Adder
  // ------
  logic adder_op_b_negate;
  logic adder_z_flag;
  logic [CVA6Cfg.XLEN:0] adder_in_a, adder_in_b;
  logic [CVA6Cfg.XLEN-1:0] adder_result;
  logic [CVA6Cfg.XLEN-1:0] operand_a_bitmanip, bit_indx;
  logic [CVA6Cfg.XLEN-1:0] operand_a_cpop;

  assign adder_op_b_negate = fu_data_i.operation inside {EQ, NE, SUB, SUBW, ANDN, ORN, XNOR};

  always_comb begin
    operand_a_bitmanip = fu_data_i.operand_a;
    operand_a_cpop     = fu_data_cpop_i.operand_a;

    if (CVA6Cfg.RVB) begin
      if (CVA6Cfg.IS_XLEN64) begin
        unique case (fu_data_i.operation)
          SH1ADDUW:    operand_a_bitmanip = fu_data_i.operand_a[31:0] << 1;
          SH2ADDUW:    operand_a_bitmanip = fu_data_i.operand_a[31:0] << 2;
          SH3ADDUW:    operand_a_bitmanip = fu_data_i.operand_a[31:0] << 3;
          CTZW:        operand_a_bitmanip = operand_a_rev32;
          ADDUW, CLZW: operand_a_bitmanip = fu_data_i.operand_a[31:0];
          CPOPW:       operand_a_cpop = fu_data_cpop_i.operand_a[31:0];
          default:     ;
        endcase
      end
      unique case (fu_data_i.operation)
        SH1ADD:  operand_a_bitmanip = fu_data_i.operand_a << 1;
        SH2ADD:  operand_a_bitmanip = fu_data_i.operand_a << 2;
        SH3ADD:  operand_a_bitmanip = fu_data_i.operand_a << 3;
        CTZ:     operand_a_bitmanip = operand_a_rev;
        default: ;
      endcase
    end
  end

  // prepare operand a
  assign adder_in_a = {operand_a_bitmanip, 1'b1};

  // prepare operand b
  assign operand_b_neg = {fu_data_i.operand_b, 1'b0} ^ {CVA6Cfg.XLEN + 1{adder_op_b_negate}};
  assign adder_in_b = operand_b_neg;

  // actual adder
  assign adder_result_ext_o = $unsigned(adder_in_a) + $unsigned(adder_in_b);
  assign adder_result = adder_result_ext_o[CVA6Cfg.XLEN:1];
  assign adder_z_flag = ~|adder_result;

endmodule
