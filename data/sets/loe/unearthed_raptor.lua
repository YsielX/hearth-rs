local function has_deathrattle(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "deathrattle" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "LOE_019",
    name = "Unearthed Raptor",
    text = "<b>Battlecry:</b> Choose a friendly minion. Gain a copy of its <b>Deathrattle</b>.",
    set = "LOE",
    type = "minion",
    class = "rogue",
    rarity = "rare",
    cost = 3,
    attack = 3,
    health = 4,
    tags = { "undead", "beast" },
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if has_deathrattle(ctx, minion) then result[#result + 1] = minion end
        end
        return result
    end,
}

function card.on_battlecry(ctx, self, target)
    if target == nil then return end
    local snapshot = ctx:entity(target)
    local copied = 0

    local function copy_if_deathrattle(card_id)
        for _, keyword in ipairs(ctx:card_definition(card_id).keywords or {}) do
            if keyword == "deathrattle" then
                ctx:attach_hook(self, "on_deathrattle", card_id)
                copied = copied + 1
                return
            end
        end
    end

    copy_if_deathrattle(snapshot.card_id)
    for _, card_id in ipairs(snapshot.attached_cards or {}) do
        copy_if_deathrattle(card_id)
    end
    for _, card_id in ipairs((snapshot.hook_attachments or {}).on_deathrattle or {}) do
        ctx:attach_hook(self, "on_deathrattle", card_id)
        copied = copied + 1
    end
    if copied > 0 then ctx:grant_keyword(self, "deathrattle") end
end

-- The attached card scripts supply the copied hooks. This no-op hook satisfies
-- the dynamically granted Deathrattle keyword's dispatch contract.
function card.on_deathrattle() end

return card
