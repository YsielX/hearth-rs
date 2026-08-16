local card = {
    api_version = 1, id = "ICC_039", name = "Dark Conviction",
    text = "Set a minion's Attack and Health to 3.", set = "ICECROWN", type = "spell",
    class = "paladin", rarity = "common", spell_school = "shadow", cost = 2,
    target_mode = "required", targets = function(ctx) return ctx:minions() end,
}

function card.on_play(ctx, self, target)
    cardlib.effects.modify_all(ctx, { target }, {
        attack = 3, health = 3, operation = "final_set", silenciable = true, reset_damage = true,
    })
end

return card
