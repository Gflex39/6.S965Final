module matrix_transpose_buffer #(
    parameter WIDTH = 12
) (
    input logic rst_in,
    input logic clk_in,

    input logic [WIDTH-1:0] in_0,
    input logic [WIDTH-1:0] in_1,
    input logic [WIDTH-1:0] in_2,
    input logic [WIDTH-1:0] in_3,
    input logic [WIDTH-1:0] in_4,
    input logic [WIDTH-1:0] in_5,
    input logic [WIDTH-1:0] in_6,
    input logic [WIDTH-1:0] in_7,

    input logic valid_in,

    output logic [WIDTH-1:0] out_0,
    output logic [WIDTH-1:0] out_1,
    output logic [WIDTH-1:0] out_2,
    output logic [WIDTH-1:0] out_3,
    output logic [WIDTH-1:0] out_4,
    output logic [WIDTH-1:0] out_5,
    output logic [WIDTH-1:0] out_6,
    output logic [WIDTH-1:0] out_7,

    output logic valid_out,
    output logic final_row_out
);

  logic [WIDTH-1:0] buffer[7:0][7:0];
  logic [2:0] row_idx, col_idx;
  logic received_last_col;
  logic clean_up;

  always_ff @(posedge clk_in) begin
    if (rst_in) begin
      row_idx <= 0;
      col_idx <= 0;
      received_last_col <= 0;
      clean_up <= 0;
    end else begin
      if (clean_up) begin
        row_idx <= 0;
        col_idx <= 0;
        received_last_col <= 0;
        clean_up <= 0;
        valid_out <= 0;
        final_row_out <= 0;
      end else begin
        if (valid_in) begin
          buffer[0][col_idx] <= in_0;
          buffer[1][col_idx] <= in_1;
          buffer[2][col_idx] <= in_2;
          buffer[3][col_idx] <= in_3;
          buffer[4][col_idx] <= in_4;
          buffer[5][col_idx] <= in_5;
          buffer[6][col_idx] <= in_6;
          buffer[7][col_idx] <= in_7;

          col_idx <= col_idx + 1;

          if (col_idx == 7) begin
            received_last_col <= 1;
          end
        end else begin
          if (received_last_col) begin
            out_0 <= buffer[row_idx][0];
            out_1 <= buffer[row_idx][1];
            out_2 <= buffer[row_idx][2];
            out_3 <= buffer[row_idx][3];
            out_4 <= buffer[row_idx][4];
            out_5 <= buffer[row_idx][5];
            out_6 <= buffer[row_idx][6];
            out_7 <= buffer[row_idx][7];

            row_idx <= row_idx + 1;

            valid_out <= 1;

            if (row_idx == 7) begin
              final_row_out <= 1;
              clean_up <= 1;
            end
          end
        end
      end
    end
  end

endmodule
