local card = {
    api_version = 1, id = "ICC_244", name = "Desperate Stand",
    text = "Give a minion \"<b>Deathrattle:</b> Return this to life with 1 Health.\"",
    set = "ICECROWN", type = "spell", class = "paladin", rarity = "rare",
    spell_school = "holy", cost = 2, target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
}

function card.on_play(ctx, self, target)
    ctx:attach_hook(target, "on_deathrattle", "ICC_244")
    cardlib.effects.grant_keyword(ctx, target, "deathrattle")
end

function card.on_deathrattle(ctx, self, position)
    cardlib.effects.summon_fresh_copy(ctx, self, position, 1, {})
end

return card
