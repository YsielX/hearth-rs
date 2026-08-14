return {
    api_version = 1, id = "AT_119", name = "Kvaldir Raider",
    text = "<b>Inspire:</b> Gain +2/+2.", set = "TGT", type = "minion", rarity = "common",
    cost = 5, attack = 4, health = 4, tags = { "undead" }, keywords = { "inspire" },
    on_inspire = function(ctx, self) ctx:buff(self, 2, 2) end,
}
