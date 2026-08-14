-- Official card: TTN_724, Storm Giant.
return {
    api_version = 1,
    id = "TTN_724",
    name = "Storm Giant",
    text = "<b>Taunt</b>\n<b>Forge:</b> Costs (2) less. Can be <b>Forged</b> endlessly.",
    set = "TITANS",
    type = "minion",
    class = "neutral",
    cost = 8,
    attack = 8,
    health = 8,
    keywords = { "taunt", "forge" },

    action_effects = {
        forge = function(ctx, self)
            ctx:modify(self, { stat = "cost", operation = "add", value = -2 })
        end,
    },
}
