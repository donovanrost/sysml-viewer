package ArithmeticBounds {
    attribute count : ScalarValues::Integer = 8;
    attribute added : ScalarValues::Integer = count + 2;
    attribute subtracted : ScalarValues::Integer = count - 3;
    attribute multiplied : ScalarValues::Integer = count * 2;
    attribute divided : ScalarValues::Integer = count / 2;
    attribute remainder : ScalarValues::Integer = count % 3;
    attribute powered : ScalarValues::Integer = 2 ^ 3;
    attribute nested : ScalarValues::Integer = (count + 2) * 3;
    attribute signed : ScalarValues::Integer = -count + 9;
    attribute large : ScalarValues::Integer = 9007199254740992 + 1;
    part add [added];
    part subtract [subtracted];
    part multiply [multiplied];
    part divide [divided];
    part modulo [remainder];
    part power [powered];
    part compound [nested];
    part unary [signed];
    part exactLarge [large];
}
