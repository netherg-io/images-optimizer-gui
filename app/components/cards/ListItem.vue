<script setup>
import { Collapse } from 'vue-collapsed';
import { formatSize } from '@/utils/helpers';

const props = defineProps({
  item: {
    type: Object,
    required: true,
  },
  level: {
    type: Number,
    default: 0,
  },
});

const filesStore = useFilesStore();

const emit = defineEmits(['remove']);

const isOpen = ref(props.item.isOpen);

const shouldRenderChildren = ref(props.item.isOpen);
let collapseTimeout = null;

watch(isOpen, (newVal) => {
  if (newVal) {
    if (collapseTimeout) clearTimeout(collapseTimeout);
    shouldRenderChildren.value = true;
  } else {
    collapseTimeout = setTimeout(() => {
      shouldRenderChildren.value = false;
    }, 300);
  }
});

const toggleFolder = () => {
  if (props.item.type === 'folder') {
    isOpen.value = !isOpen.value;
  }
};
</script>

<template>
  <div class="ui-list-item">
    <div
      class="ui-list-item__row"
      :class="{ [`ui-list-item__row--${item.type}`]: !!item.type }"
      :style="{ '--level': props.level }"
      @click="toggleFolder"
    >
      <template v-if="item.type === 'folder'">
        <CIcon
          name="chevron"
          class="ui-list-item__arrow"
          :class="{ 'ui-list-item__arrow--open': isOpen }"
        />

        <div class="ui-list-item__folder-wrapper">
          <CIcon name="folder" class="ui-list-item__folder" />
        </div>
      </template>

      <template v-else>
        <UiThumbnail :src="item.path" class="ui-list-item__image" />
      </template>

      <div class="ui-list-item__info">
        <div class="ui-list-item__name">
          <p class="s1-r">{{ item.name }}</p>
        </div>

        <div class="ui-list-item__meta">
          <template v-if="item.type === 'folder'">
            <div>
              <p class="s1-r">
                {{ $t('common.plurals.files', { count: item.fileCount }) }}
              </p>
            </div>

            <div class="ui-list-item__meta-divider"></div>
          </template>

          <p class="s1-r">{{ formatSize(item.size) }}</p>

          <template v-if="item.type === 'file'">
            <div class="ui-list-item__meta-divider"></div>

            <div>
              <p class="s1-r uppercase">{{ item.extension }}</p>
            </div>
          </template>
        </div>
      </div>

      <UiButton
        class="ui-list-item__delete"
        icon="trash"
        size="xs"
        theme="warn"
        @click.stop="emit('remove', item.id)"
      />
    </div>

    <Collapse
      :when="item.type === 'folder' && isOpen"
      class="ui-list-item__collapse"
    >
      <ASmoothList
        v-if="shouldRenderChildren"
        tag="div"
        class="ui-list-item__children"
      >
        <CardListItem
          v-for="child in item.children"
          :key="child.id"
          :item="child"
          :level="level + 1"
          @remove="filesStore.removeById"
        />
      </ASmoothList>
    </Collapse>
  </div>
</template>

<style lang="scss" scoped>
.ui-list-item {
  &__row {
    display: flex;
    gap: em(12);
    align-items: center;
    padding: em(12);
    margin-left: calc(em(24) * var(--level, 0));
    border: 1px solid $border-color-secondary;
    border-radius: em(14);
    transition: $time-normal $ease;
    transition-property: border-color, background-color;

    &--folder {
      cursor: pointer;

      @include hover {
        background-color: $background-color-tertiary;
      }
    }
  }

  &__delete {
    flex-shrink: 0;
    margin-left: auto;
  }

  &__arrow {
    flex-shrink: 0;
    width: em(12);
    height: em(7);
    rotate: 90deg;
    transition: rotate $time-fast $ease;

    &--open {
      rotate: 180deg;
    }
  }

  &__folder-wrapper {
    flex-shrink: 0;
    padding: em(10);
    background-color: $background-color-additional;
    border-radius: em(10);
    transition: background-color $time-normal $ease;
  }

  &__folder {
    width: em(20);
    height: em(20);
    color: $icon-color-additional;
    transition: color $time-normal $ease;
  }

  &__image {
    flex-shrink: 0;
    width: em(64);
    height: em(64);
    overflow: hidden;
    object-fit: cover;
    border-radius: em(10);
  }

  &__info {
    display: flex;
    flex-direction: column;
    gap: em(4);
  }

  &__name {
    & > p {
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
  }

  &__meta {
    display: flex;
    gap: em(12);
    align-items: center;
    color: $text-color-secondary;
    transition: color $time-normal $ease;
  }

  &__meta-divider {
    flex-shrink: 0;
    width: em(4);
    height: em(4);
    background-color: $border-color-accent;
    border-radius: 50%;
    transition: background-color $time-normal $ease;
  }

  &__children {
    display: flex;
    flex-direction: column;
    gap: em(12);
    margin-top: em(12);
  }

  &__collapse {
    opacity: 1;
    transition:
      height $time-normal $ease-in-out,
      opacity $time-fast $ease-in-out;

    &[data-collapse='collapsing'] {
      opacity: 0;
    }

    &[data-collapse='collapsed'] {
      opacity: 0;
    }
  }
}
</style>
