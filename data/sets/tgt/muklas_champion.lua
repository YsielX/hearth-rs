return {
    api_version = 1, id = "AT_090", name = "Mukla's Champion",
    text = "<b>Inspire:</b> Give your other minions +1/+1.",
    set = "TGT", type = "minion", rarity = "common", cost = 5, attack = 5, health = 3,
    tags = { "beast" }, keywords = { "inspire" },
    on_inspire = function(ctx, self)
        for _, minion in ipairs(ctx:board(ctx:controller(self))) do
            if minion ~= self then cardlib.effects.buff(ctx, minion, 1, 1) end
        end
    end,
}
