module idct_2d #(
    parameter WIDTH = 12
) (
    input logic rst_in,
    input logic clk_in,

    input logic [WIDTH-1:0] idct_in_0,
    input logic [WIDTH-1:0] idct_in_1,
    input logic [WIDTH-1:0] idct_in_2,
    input logic [WIDTH-1:0] idct_in_3,
    input logic [WIDTH-1:0] idct_in_4,
    input logic [WIDTH-1:0] idct_in_5,
    input logic [WIDTH-1:0] idct_in_6,
    input logic [WIDTH-1:0] idct_in_7,

    input logic valid_in,

    output logic [WIDTH-4-1:0] idct_out_0,
    output logic [WIDTH-4-1:0] idct_out_1,
    output logic [WIDTH-4-1:0] idct_out_2,
    output logic [WIDTH-4-1:0] idct_out_3,
    output logic [WIDTH-4-1:0] idct_out_4,
    output logic [WIDTH-4-1:0] idct_out_5,
    output logic [WIDTH-4-1:0] idct_out_6,
    output logic [WIDTH-4-1:0] idct_out_7,

    output logic valid_out,
    output logic final_out
);
  logic signed [WIDTH-1:0]
      temp_idct_1d_out_0,
      temp_idct_1d_out_1,
      temp_idct_1d_out_2,
      temp_idct_1d_out_3,
      temp_idct_1d_out_4,
      temp_idct_1d_out_5,
      temp_idct_1d_out_6,
      temp_idct_1d_out_7;
  logic signed [WIDTH-1:0]
      temp_idct_1d_in_0,
      temp_idct_1d_in_1,
      temp_idct_1d_in_2,
      temp_idct_1d_in_3,
      temp_idct_1d_in_4,
      temp_idct_1d_in_5,
      temp_idct_1d_in_6,
      temp_idct_1d_in_7;
  logic [WIDTH-1:0]
      mtb_out_0, mtb_out_1, mtb_out_2, mtb_out_3, mtb_out_4, mtb_out_5, mtb_out_6, mtb_out_7;
  logic mtb_valid_in, mtb_valid_out, mtb_final_row_out;

  idct_1d idct_1d_inst_0 (
      .rst_in(rst_in),
      .clk_in(clk_in),

      .idct_in_0(temp_idct_1d_in_0),
      .idct_in_1(temp_idct_1d_in_1),
      .idct_in_2(temp_idct_1d_in_2),
      .idct_in_3(temp_idct_1d_in_3),
      .idct_in_4(temp_idct_1d_in_4),
      .idct_in_5(temp_idct_1d_in_5),
      .idct_in_6(temp_idct_1d_in_6),
      .idct_in_7(temp_idct_1d_in_7),

      .idct_out_0(temp_idct_1d_out_0),
      .idct_out_1(temp_idct_1d_out_1),
      .idct_out_2(temp_idct_1d_out_2),
      .idct_out_3(temp_idct_1d_out_3),
      .idct_out_4(temp_idct_1d_out_4),
      .idct_out_5(temp_idct_1d_out_5),
      .idct_out_6(temp_idct_1d_out_6),
      .idct_out_7(temp_idct_1d_out_7)
  );

  matrix_transpose_buffer matrix_transpose_buffer_inst (
      .rst_in(rst_in),
      .clk_in(clk_in),

      .in_0(temp_idct_1d_out_0),
      .in_1(temp_idct_1d_out_1),
      .in_2(temp_idct_1d_out_2),
      .in_3(temp_idct_1d_out_3),
      .in_4(temp_idct_1d_out_4),
      .in_5(temp_idct_1d_out_5),
      .in_6(temp_idct_1d_out_6),
      .in_7(temp_idct_1d_out_7),

      .valid_in(mtb_valid_in),

      // TODO: Fix this
      .out_0(mtb_out_0),
      .out_1(mtb_out_1),
      .out_2(mtb_out_2),
      .out_3(mtb_out_3),
      .out_4(mtb_out_4),
      .out_5(mtb_out_5),
      .out_6(mtb_out_6),
      .out_7(mtb_out_7),

      .valid_out(mtb_valid_out),
      .final_row_out(mtb_final_row_out)
  );

  always_comb begin : assign_outputs
    idct_out_0 = temp_idct_1d_out_0+128;
    idct_out_1 = temp_idct_1d_out_1+128;
    idct_out_2 = temp_idct_1d_out_2+128;
    idct_out_3 = temp_idct_1d_out_3+128;
    idct_out_4 = temp_idct_1d_out_4+128;
    idct_out_5 = temp_idct_1d_out_5+128;
    idct_out_6 = temp_idct_1d_out_6+128;
    idct_out_7 = temp_idct_1d_out_7+128;
  end

  always_ff @(posedge clk_in) begin
    if (rst_in) begin
      valid_out <= 0;
      final_out <= 0;
    end else begin
      if (valid_in) begin
        temp_idct_1d_in_0 <= idct_in_0;
        temp_idct_1d_in_1 <= idct_in_1;
        temp_idct_1d_in_2 <= idct_in_2;
        temp_idct_1d_in_3 <= idct_in_3;
        temp_idct_1d_in_4 <= idct_in_4;
        temp_idct_1d_in_5 <= idct_in_5;
        temp_idct_1d_in_6 <= idct_in_6;
        temp_idct_1d_in_7 <= idct_in_7;
      end else begin
        if (final_out) begin
          final_out <= 0;
          valid_out <= 0;
        end else begin
          if (mtb_valid_out) begin
            temp_idct_1d_in_0 <= mtb_out_0;
            temp_idct_1d_in_1 <= mtb_out_1;
            temp_idct_1d_in_2 <= mtb_out_2;
            temp_idct_1d_in_3 <= mtb_out_3;
            temp_idct_1d_in_4 <= mtb_out_4;
            temp_idct_1d_in_5 <= mtb_out_5;
            temp_idct_1d_in_6 <= mtb_out_6;
            temp_idct_1d_in_7 <= mtb_out_7;

            valid_out <= mtb_valid_out;

            if (mtb_final_row_out) begin
              final_out <= 1;
            end
          end
        end
      end
      mtb_valid_in <= valid_in;
    end

  end

endmodule
