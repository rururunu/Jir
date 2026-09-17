using System;
using System.Drawing;
using System.Windows.Forms;

namespace JirDpi
{
    /// <summary>
    /// The two installer windows are laid out in 96-DPI pixels. Windows renders the
    /// text at the display's DPI but leaves those coordinates alone, so on a 150%
    /// screen the text outgrows its container and is clipped. Everything geometric
    /// is grown through here instead, and at 100% scaling every method returns its
    /// input unchanged.
    /// </summary>
    public static class DpiLayout
    {
        private static readonly float Factor = Detect();

        /// <summary>A 96-DPI length, in the display's pixels.</summary>
        public static int Px(int value)
        {
            return (int)Math.Round(value * Factor);
        }

        /// <summary>A 96-DPI length, for the pen widths that are measured in pixels.</summary>
        public static float Pen(float value)
        {
            return value * Factor;
        }

        /// <summary>A 96-DPI point, in the display's pixels.</summary>
        public static Point At(int x, int y)
        {
            return new Point(Px(x), Px(y));
        }

        /// <summary>A 96-DPI size, in the display's pixels.</summary>
        public static Size Box(int width, int height)
        {
            return new Size(Px(width), Px(height));
        }

        /// <summary>
        /// Grows a form and every control in it from the 96-DPI layout to the
        /// display's DPI.
        /// </summary>
        /// <remarks>
        /// Fonts are deliberately left alone. They are sized in points, so Windows
        /// already draws them 1.5x larger at 150%; scaling them here as well would
        /// overflow every label. This is also why WinForms' own AutoScaleMode cannot
        /// be used: it runs when the handle is created, which is after this
        /// constructor, and by then it has already read a 96 DPI baseline and
        /// concluded nothing needs to change.
        /// </remarks>
        public static void Scale(Control control)
        {
            if (Factor == 1f)
            {
                return;
            }

            foreach (Control child in control.Controls)
            {
                child.Location = At(child.Left, child.Top);
                child.Size = Box(child.Width, child.Height);
                Scale(child);
            }

            Form form = control as Form;
            if (form != null)
            {
                form.ClientSize = Box(form.ClientSize.Width, form.ClientSize.Height);
            }
        }

        private static float Detect()
        {
            // The screen DC reports the system DPI and needs no window, which is the
            // point: this runs before any handle exists. A process that is not
            // DPI-aware reads 96 here, and Windows stretches its output for it.
            using (Graphics screen = Graphics.FromHwnd(IntPtr.Zero))
            {
                return screen.DpiX / 96f;
            }
        }
    }
}
