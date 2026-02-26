class MainPage:
    def __init__(self):
        self.count = 0
        self.InitializeComponent()

    def OnCounterClicked(self, sender, e):
        self.count += 1
        if self.count == 1:
            self.CounterBtn.Text = f"Clicked {self.count} time"
        else:
            self.CounterBtn.Text = f"Clicked {self.count} times"
