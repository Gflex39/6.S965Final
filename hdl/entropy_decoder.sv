module entropy_decoder#(
    parameter DELTA_DECODE = 0
)
(
    input wire clk_in,
    input wire rst_in,
    input wire [10:0] value_in,
    input wire [5:0] run_in,
    input wire [4:0] size_in,
    input wire valid_in,
    input wire dc_in,

    output logic signed [11:0] value_out,
    output logic [5:0] run_out,
    output logic valid_out
);
    logic signed [11:0] value;
    logic signed [11:0] last_dc_value;

    always_ff @(posedge clk_in) begin
        if (valid_in && ~rst_in) begin
            value = (value_in >= (1 << (size_in - 1))) ? value_in : (value_in - (1 << size_in) + 1);
//            value = (dc_in && DELTA_DECODE) ? (value + last_dc_value) : value;
            value_out <= value;
            run_out <= run_in;
            valid_out <= 1;
//            last_dc_value <= (dc_in && DELTA_DECODE) ? value : last_dc_value;
        end else begin
            value_out <= 0;
            run_out <= 0;
            valid_out <= 0;
//            last_dc_value <= 0;
        end
    end
endmodule