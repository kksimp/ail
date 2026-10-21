from datetime import datetime, timedelta
d = datetime.fromisoformat("2026-10-21T05:00:00")
print((d + timedelta(days=3)).isoformat())
