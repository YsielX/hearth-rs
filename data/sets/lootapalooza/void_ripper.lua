local card = {
    api_version = 1,
    id = "LOOT_529",
    name = "Void Ripper",
    text = "<b>Battlecry:</b> Swap the\nAttack and Health of all other minions.",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "epic",
    cost = 3,
    attack = 3,
    health = 3,
    tags = { "demon" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local modifications = {}
    for _, minion in ipairs(ctx:minions()) do
        if minion ~= self then
            local dormant = false
            for _, keyword in ipairs(ctx:entity(minion).keywords or {}) do
                if keyword == "dormant" then dormant = true; break end
            end
            if not dormant then
                local entity = ctx:entity(minion)
                modifications[#modifications + 1] = {
                    target = minion,
                    attack = entity.health,
                    health = entity.attack,
                    reset_damage = true,
                }
            end
        end
    end
    if #modifications > 0 then ctx:modify_batch(modifications) end
end

return card
