module huffman_dc_lut(
    input wire clk_in,
    input wire rst_in,
    input wire enable_in,
    input wire [10:0] code_in,
    input wire [4:0]  code_len_in,

    output logic valid_out,
    output logic [4:0] codesize_out,
    output logic [4:0] size_out
);
    logic [4:0]     lookup_size;
    logic [4:0]     lookup_codesize;

    logic [10:0]    code;

    assign code = code_in & ((1 << code_len_in) - 1);

    always_ff @(posedge clk_in) begin
        if (rst_in) begin
            valid_out <= 0;
            codesize_out <= 0;
            size_out <= 0;
        end else begin
            case (code)
                11'b00000000000: begin lookup_size = 0;  lookup_codesize = 2; end
                11'b00000000010: begin lookup_size = 1;  lookup_codesize = 3; end
                11'b00000000011: begin lookup_size = 2;  lookup_codesize = 3; end
                11'b00000000100: begin lookup_size = 3;  lookup_codesize = 3; end
                11'b00000000101: begin lookup_size = 4;  lookup_codesize = 3; end
                11'b00000000110: begin lookup_size = 5;  lookup_codesize = 3; end
                11'b00000001110: begin lookup_size = 6;  lookup_codesize = 4; end
                11'b00000011110: begin lookup_size = 7;  lookup_codesize = 5; end
                11'b00000111110: begin lookup_size = 8;  lookup_codesize = 6; end
                11'b00001111110: begin lookup_size = 9;  lookup_codesize = 7; end
                11'b00011111110: begin lookup_size = 10; lookup_codesize = 8; end
                11'b00111111110: begin lookup_size = 11; lookup_codesize = 9; end
                default:         begin lookup_size = 0;  lookup_codesize = 0; end
            endcase

            if (enable_in && lookup_codesize != 0 && code_len_in == lookup_codesize) begin
                valid_out <= 1;
                codesize_out <= lookup_codesize;
                size_out <= lookup_size;
            end else begin
                valid_out <= 0;
                codesize_out <= 0;
                size_out <= 0;
            end
        end
    end

endmodule