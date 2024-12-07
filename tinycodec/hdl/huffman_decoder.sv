module dc_lut(
    input wire clk_in,
    input wire rst_in,
    input wire valid_in,
    input wire [10:0] code_in,
    input wire [3:0]  code_len_in,

    output logic valid_out,
    output logic [3:0] codesize_out,
    output logic [3:0] size_out
);
    logic [3:0]     lookup_size;
    logic [3:0]     lookup_codesize;
    logic           lookup_valid;

    logic [10:0]    code;
    logic [3:0]     code_len;
    logic           code_valid;

    always_ff @(posedge clk_in) begin
        if (rst_in) begin
            lookup_valid <= 0;
            lookup_size <= 0;
            lookup_codesize <= 0;
            code <= 0;
            code_len <= 0;
        end else begin
            if (valid_in) begin
                code <= code_in & ((1 << code_len_in) - 1);
                code_len <= code_len_in;
                code_valid <= 1;
            end else begin
                code_valid <= 0;
            end

            lookup_valid = 0;

            case (code)
                11'b00000000000: begin lookup_size = 0;  lookup_codesize = 2; lookup_valid = 1; end
                11'b00000000010: begin lookup_size = 1;  lookup_codesize = 3; lookup_valid = 1; end
                11'b00000000011: begin lookup_size = 2;  lookup_codesize = 3; lookup_valid = 1; end
                11'b00000000100: begin lookup_size = 3;  lookup_codesize = 3; lookup_valid = 1; end
                11'b00000000101: begin lookup_size = 4;  lookup_codesize = 3; lookup_valid = 1; end
                11'b00000000110: begin lookup_size = 5;  lookup_codesize = 3; lookup_valid = 1; end
                11'b00000001110: begin lookup_size = 6;  lookup_codesize = 4; lookup_valid = 1; end
                11'b00000011110: begin lookup_size = 7;  lookup_codesize = 5; lookup_valid = 1; end
                11'b00000111110: begin lookup_size = 8;  lookup_codesize = 6; lookup_valid = 1; end
                11'b00001111110: begin lookup_size = 9;  lookup_codesize = 7; lookup_valid = 1; end
                11'b00011111110: begin lookup_size = 10; lookup_codesize = 8; lookup_valid = 1; end
                11'b00111111110: begin lookup_size = 11; lookup_codesize = 9; lookup_valid = 1; end
            endcase

            if (lookup_valid && code_len == lookup_codesize) begin
                valid_out <= code_valid;
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

module huffman_decoder(
    input wire clk_in,
    input wire rst_in,
    input wire serial_in,
    input wire valid_in,

    output logic [11:0] value_out,
    output logic [3:0] run_out,
    output logic valid_out
);
    parameter S_DC_SIZE = 0;
    parameter S_DC_VALUE = 1;
    parameter S_AC_SIZE = 2;
    parameter S_AC_VALUE = 3;
    parameter S_RESET = 4;

    logic [2:0]     state;

    logic [26:0]    buffer;
    logic [26:0]    next_buffer;
    logic [3:0]     buffer_len;
    logic [3:0]     next_buffer_len;

    logic           mdl_valid;
    logic [3:0]     mdl_codesize;
    logic [3:0]     mdl_size;

    logic [3:0]     dc_value_size;

    dc_lut mdl(
        .clk_in(clk_in),
        .rst_in(rst_in),
        .valid_in(state == S_DC_SIZE),
        .code_in(buffer[10:0]),
        .code_len_in(buffer_len),
        .valid_out(mdl_valid),
        .codesize_out(mdl_codesize),
        .size_out(mdl_size)
    );

    always_ff @(posedge clk_in) begin
        if (rst_in || state == S_RESET) begin
            buffer <= 0;
            buffer_len <= 0;
            state <= S_DC_SIZE;
            value_out <= 0;
            valid_out <= 0;
            run_out <= 0;
            dc_value_size <= 0;
        end else begin
            next_buffer_len = buffer_len;
            next_buffer = buffer;

            case (state)
                S_DC_SIZE: begin
                    if (mdl_valid) begin
                        state <= S_DC_VALUE;
                        dc_value_size <= mdl_size;
                        next_buffer_len = next_buffer_len - mdl_codesize;
                    end else if (next_buffer_len == 15) begin
                        state <= S_RESET;
                    end
                    valid_out <= 0;
                end

                S_DC_VALUE: begin
                    if (next_buffer_len == dc_value_size) begin
                        state <= S_DC_SIZE; // TODO: changeme later to go to AC
                        run_out <= 0;
                        valid_out <= 1;
                        next_buffer_len = next_buffer_len - dc_value_size;
                        value_out <= buffer[11:0] & ((1 << dc_value_size) - 1);
                    end else valid_out <= 0;
                end
            endcase

            if (valid_in) begin
                next_buffer = {next_buffer[25:0], serial_in};
                next_buffer_len = next_buffer_len + 1;
            end

            buffer_len <= next_buffer_len;
            buffer <= next_buffer;
        end
    end

endmodule