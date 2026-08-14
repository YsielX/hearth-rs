return {
    api_version = 1,
    id = "AT_082",
    name = "Lowly Squire",
    text = "<b>Inspire:</b> Gain +1 Attack.",
    set = "TGT",
    type = "minion",
    cost = 1,
    attack = 1,
    health = 2,
    keywords = { "inspire" },
    on_inspire = function(ctx, self)
        ctx:buff(self, 1, 0)
    end,
}
