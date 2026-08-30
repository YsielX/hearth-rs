return {
    api_version = 1, id = "AT_089", name = "Boneguard Lieutenant",
    text = "<b>Inspire:</b> Gain +1 Health.",
    set = "TGT", type = "minion", rarity = "common", cost = 2, attack = 3, health = 2,
    tags = { "undead" }, keywords = { "inspire" },
    on_inspire = function(ctx, self) cardlib.effects.buff(ctx, self, 0, 1) end,
}
