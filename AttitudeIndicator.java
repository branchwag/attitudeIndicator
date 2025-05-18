import javax.swing.*;
import java.awt.*;
import java.awt.event.*;
import java.awt.geom.*;

public class AttitudeIndicator extends JFrame {
	private AttitudePanel attitudePanel;
	private JSlider pitchSlider;
	private JSlider rollSlider;

	public AttitudeIndicator() {
		super("Aircraft Attitude Indicator");
		setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
		setSize(500, 600);
		// https://docs.oracle.com/javase/7/docs/api/java/awt/BorderLayout.html
		setLayout(new BorderLayout());

		attitudePanel = new AttitudePanel();
		add(attitudePanel, BorderLayout.CENTER);

		JPanel controlPanel = new JPanel();
		controlPanel.setLayout(new GridLayout(2, 2)); // row, col

		JLabel pitchLabel = new JLabel("Pitch: 0°", JLabel.CENTER);
		pitchSlider = new JSlider(JSlider.HORIZONTAL, -90, 90, 0);
		pitchSlider.setMajorTickSpacing(30);
		pitchSlider.setMinorTickSpacing(10);
		pitchSlider.setPaintTicks(true);
		pitchSlider.setPaintLabels(true);
		pitchSlider.addChangeListener(e -> {
			int value = pitchSlider.getValue();
			pitchLabel.setText("Pitch: " + value + "°");
			attitudePanel.setPitch(value);
		});

		JLabel rollLabel = new JLabel("Roll: 0°", JLabel.CENTER);
		rollSlider = new JSlider(JSlider.HORIZONTAL, -180, 180, 0);
		rollSlider.setMajorTickSpacing(60);
		rollSlider.setMinorTickSpacing(15);
		rollSlider.setPaintTicks(true);
		rollSlider.setPaintLabels(true);
		rollSlider.addChangeListener(e -> {
			int value = rollSlider.getValue();
			rollLabel.setText("Roll: " + value + "°");
			attitudePanel.setRoll(value);
		});

		controlPanel.add(pitchLabel);
		controlPanel.add(rollLabel);
		controlPanel.add(pitchSlider);
		controlPanel.add(rollSlider);

		add(controlPanel, BorderLayout.SOUTH);

		setLocationRelativeTo(null);
	}

	public static void main(String[] args) {
		new AttitudeIndicator().setVisible(true);
	}

	class AttitudePanel extends JPanel {
		private double pitch = 0.0; // degrees, -90 to 90
		private double roll = 0.0; // degrees, -180 to 180

		private final Color SKY_COLOR = new Color(0, 153, 255);
		private final Color GROUND_COLOR = new Color(153, 102, 51);
		private final Color MARKINGS_COLOR = Color.WHITE;
		private final Color PLANE_SYMBOL_COLOR = Color.YELLOW; // maybe black later if yellow looks weird?

		public AttitudePanel() {
			setPreferredSize(new Dimension(400, 400));
			setBackground(Color.BLACK);
		}

		public void setPitch(double pitch) {
			this.pitch = Math.max(-90, Math.min(90, pitch));
			repaint();
		}

		public void setRoll(double roll) {
			this.roll = roll;
			repaint();
		}

		@Override
		protected void paintComponent(Graphics g) {
			super.paintComponent(g);
			Graphics2D g2d = (Graphics2D) g.create();
			g2d.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);

			int width = getWidth();
			int height = getHeight();
			int centerX = width / 2;
			int centerY = height / 2;

			// circular clip for the indicator
			int diameter = Math.min(width, height) - 40;
			int radius = diameter / 2;
			Ellipse2D.Double clipCircle = new Ellipse2D.Double(centerX - radius, centerY - radius, diameter,
					diameter);
			g2d.setClip(clipCircle);

			// horizon transformation
			AffineTransform originalTransform = g2d.getTransform();
			g2d.translate(centerX, centerY);
			g2d.rotate(Math.toRadians(roll));

			// how much to shift horizon based on pitch
			// degree = 4 pixels
			int pitchPixels = (int) (pitch * 4);

			// drawing
			g2d.setColor(SKY_COLOR);
			g2d.fillRect(-radius, -radius - pitchPixels, diameter, radius);
			g2d.setColor(GROUND_COLOR);
			g2d.fillRect(-radius, -pitchPixels, diameter, radius);

			// horizon line
			g2d.setColor(MARKINGS_COLOR);
			g2d.setStroke(new BasicStroke(3));
			g2d.drawLine(-radius, -pitchPixels, radius, -pitchPixels);

			// pitch markings every 10 degrees
			g2d.setStroke(new BasicStroke(2));
			for (int i = -9; i <= 9; i++) {
				if (i == 0)
					continue;

				// 40 pixels = 10 degrees
				int y = -pitchPixels - (i * 40);
				// longer lines for 30 degree marks
				int lineLength = (i % 3 == 0) ? 60 : 30;

				g2d.drawLine(-lineLength, y, lineLength, y);

				// pitch value
				String pitchText = Math.abs(i * 10) + "°";
				FontMetrics fm = g2d.getFontMetrics();
				int textWidth = fm.stringWidth(pitchText);
				g2d.drawString(pitchText, lineLength + 5, y + fm.getAscent() / 2);
				g2d.drawString(pitchText, -lineLength - 5 - textWidth, y + fm.getAscent() / 2);
			}

			// reset tranform for fixed aircraft symbol
			g2d.setTransform(originalTransform);

			// draw outer bezel
			g2d.setColor(Color.DARK_GRAY);
			g2d.setStroke(new BasicStroke(4));
			g2d.draw(clipCircle);

			// draw fixed roll indicator at top
			int triangleSize = 10;
			g2d.setColor(Color.WHITE);
			int[] xPoints = { centerX, centerX - triangleSize, centerX + triangleSize };
			int[] yPoints = { centerY - radius + 5, centerY - radius + triangleSize + 5,
					centerY - radius + triangleSize + 5 };
			g2d.fillPolygon(xPoints, yPoints, 3);

			// draw roll markers on bezel
			g2d.setFont(new Font("Arial", Font.BOLD, 12));
			for (int angle = 0; angle < 360; angle += 30) {
				if (angle == 0) {
					// Special marker for 0 degrees (upright)
					int markerLength = 15;
					double rads = Math.toRadians(angle - 90); // -90 to start from the top
					int x1 = centerX + (int) ((radius - 5) * Math.cos(rads));
					int y1 = centerY + (int) ((radius - 5) * Math.sin(rads));
					int x2 = centerX + (int) ((radius - markerLength) * Math.cos(rads));
					int y2 = centerY + (int) ((radius - markerLength) * Math.sin(rads));
					g2d.setStroke(new BasicStroke(3));
					g2d.drawLine(x1, y1, x2, y2);
				} else {
					// Regular roll markers
					int markerLength = (angle % 90 == 0) ? 15 : 10;
					double rads = Math.toRadians(angle - 90); // -90 to start from the top
					int x1 = centerX + (int) ((radius - 5) * Math.cos(rads));
					int y1 = centerY + (int) ((radius - 5) * Math.sin(rads));
					int x2 = centerX + (int) ((radius - markerLength) * Math.cos(rads));
					int y2 = centerY + (int) ((radius - markerLength) * Math.sin(rads));
					g2d.setStroke(new BasicStroke(2));
					g2d.drawLine(x1, y1, x2, y2);

					// Draw angle labels for 30 degree increments
					if (angle > 0 && angle < 180) {
						String text = Integer.toString(angle);
						FontMetrics fm = g2d.getFontMetrics();
						int textWidth = fm.stringWidth(text);
						int textX = centerX
								+ (int) ((radius - markerLength - 20) * Math.cos(rads))
								- textWidth / 2;
						int textY = centerY
								+ (int) ((radius - markerLength - 20) * Math.sin(rads))
								+ fm.getAscent() / 2;
						g2d.drawString(text, textX, textY);
					} else if (angle > 180) {
						String text = Integer.toString(angle - 360); // Show negative for left
												// side
						FontMetrics fm = g2d.getFontMetrics();
						int textWidth = fm.stringWidth(text);
						int textX = centerX
								+ (int) ((radius - markerLength - 20) * Math.cos(rads))
								- textWidth / 2;
						int textY = centerY
								+ (int) ((radius - markerLength - 20) * Math.sin(rads))
								+ fm.getAscent() / 2;
						g2d.drawString(text, textX, textY);
					}
				}
			}

			// Remove clip for drawing aircraft symbol
			g2d.setClip(null);

			// Draw fixed aircraft symbol
			g2d.setColor(PLANE_SYMBOL_COLOR);
			g2d.setStroke(new BasicStroke(3));

			// Horizontal line (wings)
			g2d.drawLine(centerX - 60, centerY, centerX + 60, centerY);

			// Center dot
			g2d.fillOval(centerX - 5, centerY - 5, 10, 10);

			// Small vertical line in center
			g2d.drawLine(centerX, centerY - 10, centerX, centerY + 10);

			g2d.dispose();
		}
	}
}
