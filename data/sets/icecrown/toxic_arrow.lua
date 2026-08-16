local card = {
    api_version = 1, id = "ICC_049", name = "Toxic Arrow",
    text = "Deal $2 damage to a minion. If it survives, give it <b>Poisonous</b>.",
    set = "ICECROWN", type = "spell", class = "hunter", rarity = "epic",
    spell_school = "nature", cost = 2, target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
}

function card.on_play(ctx, self, target)
    cardlib.effects.damage(ctx, target, 2)
    ctx:continue_with_entity("toxic_arrow_survived", target)
end

function card.toxic_arrow_survived(ctx, self, target)
    if ctx:entity(target).zone == "board" then ctx:grant_keyword(target, "poisonous") end
end

return card
