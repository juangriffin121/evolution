use crate::mods::blobs::Blob;
pub fn matrix_prod(matrix: &Vec<Vec<f32>>, vector: &Vec<f32>) -> Vec<f32> {
    let mut output: Vec<f32> = Vec::new();
    for i in 0..matrix.len() {
        let mut s = 0.0;
        for k in 0..vector.len() {
            s += matrix[i][k] * vector[k];
        }
        output.push(s);
    }
    output
}

pub fn matrix_sum(matrix1: &Vec<Vec<f32>>, matrix2: &Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    // Check if dimensions match
    if matrix1.len() != matrix2.len() || matrix1[0].len() != matrix2[0].len() {
        panic!("Matrices dimensions do not match");
    }

    let mut output: Vec<Vec<f32>> = Vec::new();

    for i in 0..matrix1.len() {
        let mut row: Vec<f32> = Vec::new();
        for j in 0..matrix1[0].len() {
            row.push(matrix1[i][j] + matrix2[i][j]);
        }
        output.push(row);
    }

    output
}

pub fn sum_weights(
    matrices1: &Vec<Vec<Vec<f32>>>,
    matrices2: &Vec<Vec<Vec<f32>>>,
) -> Vec<Vec<Vec<f32>>> {
    // Check if both vectors have the same number of matrices
    if matrices1.len() != matrices2.len() {
        panic!("The vectors of matrices must have the same length");
    }

    // Initialize the output vector of matrices
    let mut sum_matrices: Vec<Vec<Vec<f32>>> = Vec::new();

    // Iterate over each matrix pair
    for (matrix1, matrix2) in matrices1.iter().zip(matrices2.iter()) {
        // Check if dimensions of each corresponding matrix pair match
        if matrix1.len() != matrix2.len() || matrix1[0].len() != matrix2[0].len() {
            panic!("All corresponding matrices must have the same dimensions");
        }

        // Initialize the sum matrix with the same dimensions as the input matrices
        let rows = matrix1.len();
        let cols = matrix1[0].len();
        let mut sum_matrix = vec![vec![0.0; cols]; rows];

        // Sum the corresponding elements of the matrices
        for i in 0..rows {
            for j in 0..cols {
                sum_matrix[i][j] = matrix1[i][j] + matrix2[i][j];
            }
        }

        // Add the sum matrix to the output vector
        sum_matrices.push(sum_matrix);
    }

    sum_matrices
}

pub fn distance_to_segment(
    object_center: &(f32, f32),
    starting_point: &(f32, f32),
    direction: &(f32, f32),
    length: f32,
) -> f32 {
    let displacement = (
        object_center.0 - starting_point.0,
        object_center.1 - starting_point.1,
    );

    let projection = displacement.0 * direction.0 + displacement.1 * direction.1;
    let projection = 0.0_f32.max(length.min(projection));
    let closest_point = (projection * direction.0, projection * direction.1);
    let distance =
        (object_center.0 - closest_point.0).powi(2) + (object_center.1 - closest_point.1).powi(2);
    if distance == 0.0 {
        return 0.0;
    }
    1.0 / distance
}

pub fn visual_neuron_activation(
    visible_blobs: &Vec<&Blob>,
    neuron_starting_point: &(f32, f32),
    direction: &(f32, f32),
    length: f32,
) -> f32 {
    let mut sum = 0.;
    for blob in visible_blobs {
        sum += distance_to_segment(&blob.position, neuron_starting_point, direction, length)
    }
    sum
}
