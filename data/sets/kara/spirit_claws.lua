local function has_spell_damage(ctx, self)
    local player = ctx:controller(self)
    if ctx:entity(ctx:player(player).hero).spell_damage > 0 then return true end
    for _, minion in ipairs(ctx:board(player)) do
        local dormant = false
        for _, keyword in ipairs(ctx:entity(minion).keywords) do
            if keyword == "dormant" then dormant = true break end
        end
        if not dormant and ctx:entity(minion).spell_damage > 0 then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "KAR_063",
    name = "Spirit Claws",
    text = "[x]Has +2 Attack while you\nhave <b>Spell Damage</b>.",
    set = "KARA",
    type = "weapon",
    class = "shaman",
    rarity = "common",
    cost = 1,
    attack = 1,
    health = 3,
    auras = {{
        active_zones = { "weapon" },
        attack = function(ctx, self) return has_spell_damage(ctx, self) and 2 or 0 end,
        targets = function(ctx, self) return { self } end,
    }},
}
