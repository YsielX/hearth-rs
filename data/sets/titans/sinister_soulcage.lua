local function is_undead(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "undead" or tag == "all" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "YOG_513",
    name = "Sinister Soulcage",
    text = "[x]Give a friendly Undead\n+2/+2. Spend 5 <b>Corpses</b>\nto summon a copy of it.",
    set = "TITANS",
    type = "spell",
    class = "death_knight",
    rarity = "rare",
    spell_school = "shadow",
    cost = 4,
    rune_cost = { unholy = 1 },
    target_mode = "required",
    targets = function(ctx, self)
        local targets = {}
        for _, entity in ipairs(ctx:friendly_minions(self)) do
            if is_undead(ctx, entity) then targets[#targets + 1] = entity end
        end
        return targets
    end,
}

function card.on_play(ctx, self, target)
    local player = ctx:controller(self)
    cardlib.effects.buff(ctx, target, 2, 2)
    if #ctx:board(player) < 7 then
        ctx:set_data(self, "soulcage_target", target)
        ctx:spend_resource_and_continue(player, "corpses", 5, 5, "summon_soulcage_copy")
    end
end

function card.summon_soulcage_copy(ctx, self, spent)
    if spent > 0 then
        ctx:summon_copy(ctx:controller(self), ctx:get_data(self, "soulcage_target"))
    end
end

return card
