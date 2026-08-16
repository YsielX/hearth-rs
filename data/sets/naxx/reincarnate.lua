local card = {
    api_version = 1,
    id = "FP1_025",
    name = "Reincarnate",
    text = "Destroy a minion,\nthen return it to life with full Health.",
    set = "NAXX",
    type = "spell",
    class = "shaman",
    rarity = "common",
    spell_school = "nature",
    cost = 1,
    target_mode = "required",
}

function card.targets(ctx, self)
    return ctx:minions()
end

function card.on_play(ctx, self, target)
    local entity = ctx:entity(target)
    cardlib.effects.destroy(ctx, target)
    ctx:continue_with_value("return_to_life", {
        card_id = entity.card_id,
        player = entity.controller,
        position = ctx:board_position(target),
    })
end

function card.return_to_life(ctx, self, dead)
    ctx:summon_at(dead.player, dead.card_id, dead.position)
end

return card
